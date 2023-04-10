use crate::client::Client;
use crate::config::Config;
use crate::{conn_message, disc_message, request_to_json, response_to_json, Result};
use chrono::Utc;
use futures::SinkExt;
use log::info;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

lazy_static::lazy_static! {
    static ref CONFIG: Config = Config::default();
}

type Clients = Arc<Mutex<HashMap<SocketAddr, mpsc::UnboundedSender<String>>>>;

async fn bind_server() -> Result<TcpListener> {
    match TcpListener::bind(CONFIG.server_address.clone()).await {
        Ok(listener) => {
            info!("Server is listening on {}", CONFIG.server_address);
            Ok(listener)
        }
        Err(_) => Err("Could not bind the server to this address".into()),
    }
}

pub async fn run() -> Result<()> {
    let listener = bind_server().await?;
    let clients = Arc::new(Mutex::new(HashMap::new()));
    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        let clients = Arc::clone(&clients);
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, addr, &clients).await {
                info!("{}", e);
            }
        });
    }
}

async fn handle_client(stream: TcpStream, addr: SocketAddr, clients: &Clients) -> Result<()> {
    let mut lines = Framed::new(stream, LinesCodec::new());

    let username = validate_username(&mut lines, addr).await?;
    let mut client = Client::new(clients, username, addr).await;

    handle_new_connection(clients, &client).await;

    loop {
        tokio::select! {
            Some(msg) = client.rx.recv() => {
                lines.send(&msg).await?;
            }
            request = lines.next() => match request {
                Some(Ok(request)) => {
                    handle_request(clients, &client, &request).await?;
                }
                Some(Err(e)) => {
                    return Err(e.into());
                }
                None => break,
            }
        }
    }

    handle_disconnection(clients, &client).await;
    Ok(())
}

async fn handle_request(clients: &Clients, client: &Client, request: &str) -> Result<()> {
    let json_request: Value = serde_json::from_str(request).unwrap();
    let message = json_request.get("body").and_then(|v| v.as_str());

    if CONFIG.is_valid_message(message, client.addr)? {
        info!("{} sent a message to the server", client.username);
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S %z").to_string();
        let request = request_to_json!(
            "SendMessage",
            json!({ "data": message.unwrap().to_string().trim(), "sender": client.username, "date": now })
        );
        broadcast(clients, client.addr, &request).await;
    } else {
        let response = response_to_json!(400, "InvalidMessage");
        send_targeted(clients, client.addr, &response).await;
    }

    Ok(())
}

async fn validate_username(
    lines: &mut Framed<TcpStream, LinesCodec>,
    client_addr: SocketAddr,
) -> Result<String> {
    loop {
        let mut username = None;
        let request = match lines.next().await {
            Some(Ok(request)) if request.is_empty() => {
                return Err(
                    format!("{} disconnected before entering username", client_addr).into(),
                );
            }
            Some(Ok(request)) => request,
            // TODO: Handle errors
            _ => unreachable!(),
        };

        let json_request: Value = serde_json::from_str(&request).unwrap();
        let (response, status_code) = match json_request.get("method").and_then(|v| v.as_str()) {
            Some("LogInUsername") => {
                username = json_request.get("body").and_then(|v| v.as_str());
                if CONFIG.is_valid_username(username, client_addr)? {
                    (response_to_json!(200, "OK"), 200)
                } else {
                    (response_to_json!(400, "InvalidUsername"), 400)
                }
            }
            _ => (response_to_json!(400, "BadRequest"), 400),
        };

        lines.send(&response).await?;
        if status_code == 200 {
            return Ok(username.unwrap().to_string());
        }
    }
}

async fn handle_new_connection(clients: &Clients, client: &Client) {
    let conn_message = conn_message!(&client.username);
    info!(
        "{} ({}) has been connected to the server",
        client.username, client.addr
    );
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S %z").to_string();
    let request = request_to_json!("Connection", json!({ "data": conn_message, "date": now }));

    broadcast(clients, client.addr, &request).await;
}

async fn handle_disconnection(clients: &Clients, client: &Client) {
    let disc_message = disc_message!(&client.username);
    info!(
        "{} ({}) has been disconnected from the server",
        client.username, client.addr
    );
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S %z").to_string();
    let request = request_to_json!("Connection", json!({ "data": disc_message, "date": now }));

    broadcast(clients, client.addr, &request).await;
}

async fn broadcast(clients: &Clients, sender: SocketAddr, request: &str) {
    let mut clients = clients.lock().await;
    for client in clients.iter_mut() {
        if *client.0 != sender {
            client.1.send(request.into()).unwrap();
        }
    }
}

// TODO: Optimize it
async fn send_targeted(clients: &Clients, target: SocketAddr, request: &str) {
    let mut clients = clients.lock().await;
    for client in clients.iter_mut() {
        if *client.0 == target {
            client.1.send(request.into()).unwrap();
        }
    }
}
