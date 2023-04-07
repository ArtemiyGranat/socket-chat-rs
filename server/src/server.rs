use crate::config::Config;
use crate::Result;
use crate::{conn_message, disc_message, request_to_json, response_to_json};
use chrono::Utc;
use log::info;
use serde_json::{json, Value};
use std::net::SocketAddr;
use tokio::io::AsyncWrite;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{tcp::ReadHalf, TcpListener, TcpStream},
    sync::broadcast::{self, Sender},
};

async fn init_server(config: &Config) -> Result<TcpListener> {
    match TcpListener::bind(config.server_address.clone()).await {
        Ok(listener) => {
            info!("Server is listening on {}", config.server_address);
            Ok(listener)
        }
        Err(_) => Err("Could not bind the server to this address".into()),
    }
}

pub async fn run_server(config: &Config) -> Result<()> {
    let listener = init_server(config).await?;
    let (sender, _) = broadcast::channel(config.max_connections);
    loop {
        let config = config.clone();
        let (client_socket, _) = listener.accept().await.unwrap();
        let mut sender = sender.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_client(&config, client_socket, &mut sender).await {
                info!("{}", e);
            }
        });
    }
}

async fn handle_client(
    config: &Config,
    mut client_socket: TcpStream,
    sender: &mut Sender<(String, Option<SocketAddr>)>,
) -> Result<()> {
    let mut receiver = sender.subscribe();
    let client_addr = client_socket.peer_addr().unwrap();
    let (reader, mut writer) = client_socket.split();
    let mut reader = BufReader::new(reader);

    let username = validate_username(config, &mut reader, &mut writer, client_addr).await?;
    let _conn_message = conn_message!(&username);
    info!(
        "{} ({}) has been connected to the server",
        username, client_addr
    );
    // TODO: Send connection message to all other clients (_conn_message should be used).
    handle_new_connection(username.clone());

    let mut request = String::new();
    loop {
        tokio::select! {
            request_size = reader.read_line(&mut request) => {
                if let Ok(0) = request_size {
                    handle_disconnection(username, sender, client_addr);
                    break Ok(())
                }
                handle_request(
                    config,
                    request_size,
                    client_addr,
                    username.clone(),
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

// TODO: Add request handling for the future
async fn handle_request<W: AsyncWrite + Unpin>(
    config: &Config,
    size: std::io::Result<usize>,
    client_addr: SocketAddr,
    username: String,
    request: &mut String,
    sender: &mut Sender<(String, Option<SocketAddr>)>,
    writer: &mut W,
) -> Result<()> {
    if let Err(e) = size {
        return Err(e.into());
    }
    let json_request: Value = serde_json::from_str(request).unwrap();
    let message = json_request.get("body").and_then(|v| v.as_str());

    if config.is_valid_message(message, client_addr)? {
        info!("{} sent a message to the server", username);
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S %z").to_string();
        let request = request_to_json!(
            "SendMessage",
            json!({ "data": message.unwrap().to_string().trim(), "sender": username, "date": now })
        );
        sender.send((request, Some(client_addr))).unwrap();
    } else {
        let response = response_to_json!(400, "InvalidMessage");
        write_data(writer, response).await?;
    }

    request.clear();
    Ok(())
}

async fn validate_username<W: AsyncWrite + Unpin>(
    config: &Config,
    reader: &mut BufReader<ReadHalf<'_>>,
    writer: &mut W,
    client_addr: SocketAddr,
) -> Result<String> {
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
                if config.is_valid_username(username, client_addr)? {
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

async fn write_data<W: AsyncWrite + Unpin>(writer: &mut W, data: String) -> Result<()> {
    writer.write_all(data.as_bytes()).await?;
    Ok(())
}
