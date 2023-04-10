use crate::client::Client;
use crate::config::Config;
use crate::{request_to_json, response_to_json, Result};
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

// TODO: Check how to deal with the private messages if you have only SocketAddr and no username
type Clients = Arc<Mutex<HashMap<SocketAddr, mpsc::UnboundedSender<String>>>>;

async fn bind_server() -> Result<TcpListener> {
    match TcpListener::bind(CONFIG.server_address.clone()).await {
        Ok(listener) => {
            info!("Server is listening on {}", CONFIG.server_address);
            Ok(listener)
        }
        Err(e) => Err(format!("Could not bind the server to this address: {e}").into()),
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
                info!("{e}");
            }
        });
    }
}

async fn handle_client(stream: TcpStream, addr: SocketAddr, clients: &Clients) -> Result<()> {
    let mut lines = Framed::new(stream, LinesCodec::new());

    let username = authorize_user(&mut lines, addr).await?;
    let mut client = Client::new(clients, username, addr).await;

    new_connection_info(clients, &client).await?;

    loop {
        tokio::select! {
            Some(msg) = client.rx.recv() => {
                if let Err(e) = lines.send(&msg).await {
                    info!("Could not send a message to {}: {e}", client.addr);
                    break;
                }
            }
            request = lines.next() => match request {
                Some(Ok(request)) => {
                    if let Err(e) = handle_request(clients, &client, &request).await {
                        info!("Error with {} occured: {e}", client.addr);
                        break;
                    }
                }
                Some(Err(e)) => {
                    info!("Invalid request from {}: {e}", client.addr);
                    break;
                }
                None => break,
            }
        }
    }

    clients.lock().await.remove(&addr);
    disconnection_info(clients, &client).await?;
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
        broadcast(clients, client.addr, &request).await?;
    } else {
        let response = response_to_json!(400, "InvalidMessage");
        send_targeted(clients, client.addr, &response).await?;
    }

    Ok(())
}

async fn authorize_user(
    lines: &mut Framed<TcpStream, LinesCodec>,
    client_addr: SocketAddr,
) -> Result<String> {
    loop {
        let mut username = None;
        let request = match lines.next().await {
            Some(Ok(request)) => request,
            Some(Err(e)) => return Err(format!("Invalid request from {client_addr}: {e}").into()),
            None => return Err(format!("{client_addr} disconnected before entering username").into()),
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

        if let Err(e) = lines.send(&response).await {
            info!("Could not send a message to {client_addr}: {e}");
        }

        if status_code == 200 {
            return Ok(username.unwrap().to_string());
        }
    }
}

async fn new_connection_info(clients: &Clients, client: &Client) -> Result<()> {
    let info = format!("{} has been connected to the server", &client.username);
    info!(
        "{} ({}) has been connected to the server",
        client.username, client.addr
    );
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S %z").to_string();
    let request = request_to_json!("Connection", json!({ "data": info, "date": now }));

    broadcast(clients, client.addr, &request).await?;
    Ok(())
}

async fn disconnection_info(clients: &Clients, client: &Client) -> Result<()> {
    let info = format!("{} has been disconnected from the server", &client.username);
    info!(
        "{} ({}) has been disconnected from the server",
        client.username, client.addr
    );
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S %z").to_string();
    let request = request_to_json!("Connection", json!({ "data": info, "date": now }));

    broadcast(clients, client.addr, &request).await?;
    Ok(())
}

async fn broadcast(clients: &Clients, sender: SocketAddr, request: &str) -> Result<()> {
    let mut clients = clients.lock().await;
    for client in clients.iter_mut() {
        if *client.0 != sender {
            if let Err(e) = client.1.send(request.into()) {
                info!("Could not send a message to {}: {e}", client.0);
            }
        }
    }
    Ok(())
}

async fn send_targeted(clients: &Clients, target: SocketAddr, request: &str) -> Result<()> {
    let mut clients = clients.lock().await;
    if let Some(client) = clients.get_mut(&target) {
        if let Err(e) = client.send(request.into()) {
            info!("Could not send a message to {target}: {e}");
        }
    } else {
        return Err(format!("Could not find a user: {}", target).into());
    }
    Ok(())
}
