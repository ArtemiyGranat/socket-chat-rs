use crate::client::Client;
use crate::config::Config;
use crate::{conn_message, disc_message, request_to_json, response_to_json, Result};
use chrono::Utc;
use log::info;
use serde_json::{json, Value};
use std::net::SocketAddr;
use tokio::io::{AsyncBufRead, AsyncWrite};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::broadcast::{self, Sender},
};

lazy_static::lazy_static! {
    static ref CONFIG: Config = Config::default();
}

async fn init_server() -> Result<TcpListener> {
    match TcpListener::bind(CONFIG.server_address.clone()).await {
        Ok(listener) => {
            info!("Server is listening on {}", CONFIG.server_address);
            Ok(listener)
        }
        Err(_) => Err("Could not bind the server to this address".into()),
    }
}

pub async fn run_server() -> Result<()> {
    let listener = init_server().await?;
    let (sender, _) = broadcast::channel(CONFIG.max_connections);
    loop {
        let (client_socket, _) = listener.accept().await.unwrap();
        let mut sender = sender.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_client(client_socket, &mut sender).await {
                info!("{}", e);
            }
        });
    }
}

async fn handle_client(
    mut client_socket: TcpStream,
    sender: &mut Sender<(String, Option<SocketAddr>)>,
) -> Result<()> {
    let mut receiver = sender.subscribe();
    let client_addr = client_socket.peer_addr().unwrap();
    let (reader, mut writer) = client_socket.split();
    let mut reader = BufReader::new(reader);

    let username = validate_username(&mut reader, &mut writer, client_addr).await?;
    let client = Client::new(username, client_addr);
    let _conn_message = conn_message!(&client.username);
    info!(
        "{} ({}) has been connected to the server",
        client.username, client_addr
    );
    // TODO: Send connection message to all other clients (_conn_message should be used).
    handle_new_connection(client.username.clone());

    let mut request = String::new();
    loop {
        tokio::select! {
            request_size = reader.read_line(&mut request) => {
                if let Ok(0) = request_size {
                    handle_disconnection(client.username, sender, client_addr);
                    break Ok(())
                }
                handle_request(
                    request_size,
                    &client,
                    &mut request,
                    sender,
                    &mut writer
                )
                .await?;
            }
            outgoing_data = receiver.recv() => {
                let (message, sender_addr) = outgoing_data.unwrap();
                if let Some(sender_addr) = sender_addr {
                    if client_addr != sender_addr {
                        write_data(&mut writer, message).await?;
                    }
                } else {
                    write_data(&mut writer, message).await?;
                }
            }
        }
    }
}

async fn handle_request<W>(
    size: std::io::Result<usize>,
    client: &Client,
    request: &mut String,
    sender: &mut Sender<(String, Option<SocketAddr>)>,
    writer: &mut W,
) -> Result<()>
where
    W: AsyncWrite + Unpin,
{
    if let Err(e) = size {
        return Err(e.into());
    }
    let json_request: Value = serde_json::from_str(request).unwrap();
    let message = json_request.get("body").and_then(|v| v.as_str());

    if CONFIG.is_valid_message(message, client.client_addr)? {
        info!("{} sent a message to the server", client.username);
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S %z").to_string();
        let request = request_to_json!(
            "SendMessage",
            json!({ "data": message.unwrap().to_string().trim(), "sender": client.username, "date": now })
        );
        sender.send((request, Some(client.client_addr))).unwrap();
    } else {
        let response = response_to_json!(400, "InvalidMessage");
        write_data(writer, response).await?;
    }

    request.clear();
    Ok(())
}

async fn validate_username<W, R>(
    reader: &mut R,
    writer: &mut W,
    client_addr: SocketAddr,
) -> Result<String>
where
    R: AsyncBufRead + Unpin,
    W: AsyncWrite + Unpin,
{
    loop {
        let mut request = String::new();
        let mut username: Option<&str> = None;

        reader.read_line(&mut request).await?;
        if request.is_empty() {
            return Err(format!("{} disconnected before entering username", client_addr).into());
        }
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

        write_data(writer, response).await?;
        if status_code == 200 {
            return Ok(username.unwrap().to_string());
        }
    }
}

fn handle_new_connection(username: String) {
    let _conn_message = conn_message!(&username);
}

fn handle_disconnection(
    username: String,
    sender: &mut Sender<(String, Option<SocketAddr>)>,
    client_addr: SocketAddr,
) {
    let disc_message = disc_message!(&username);
    info!(
        "{} ({}) has been disconnected from the server",
        username, client_addr
    );
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S %z").to_string();
    let request = request_to_json!("Connection", json!({ "data": disc_message, "date": now }));
    sender.send((request, Some(client_addr))).unwrap();
}

async fn write_data<W>(writer: &mut W, data: String) -> Result<()>
where
    W: AsyncWrite + Unpin,
{
    writer.write_all(data.as_bytes()).await?;
    Ok(())
}
