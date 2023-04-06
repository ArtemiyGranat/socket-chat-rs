use crate::config::Config;
use crate::Result;
use crate::{conn_message, disc_message, request_to_json, response_to_json};
use chrono::{Local, Utc};
use log::info;
use serde_json::Value;
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

    let mut data = String::new();
    loop {
        tokio::select! {
            received_data_size = reader.read_line(&mut data) => {
                if let Ok(0) = received_data_size {
                    handle_disconnection(username, sender, client_addr);
                    break Ok(())
                }
                handle_received_data(
                    config,
                    received_data_size,
                    client_addr,
                    username.clone(),
                    &mut data,
                    sender,
                    &mut writer
                )
                .await?;
            }
            outgoing_data = receiver.recv() => {
                let (message, sender_addr) = outgoing_data.unwrap();
                if let Some(sender_addr) = sender_addr {
                    if client_addr != sender_addr {
                        writer.write_all(message.as_bytes()).await.unwrap();
                    }
                } else {
                    writer.write_all(message.as_bytes()).await.unwrap();
                }
            }
        }
    }
}

// TODO: Add request handling for the future
async fn handle_received_data<W: AsyncWrite + Unpin>(
    config: &Config,
    size: std::io::Result<usize>,
    client_addr: SocketAddr,
    username: String,
    data: &mut String,
    sender: &mut Sender<(String, Option<SocketAddr>)>,
    writer: &mut W,
) -> Result<()> {
    if let Err(e) = size {
        return Err(e.into());
    }
    let json_data: Value = serde_json::from_str(data).unwrap();
    let msg = json_data
        .get("body")
        .and_then(|v| v.as_str())
        .unwrap()
        .to_string();
    if config.is_valid_message(msg.trim()) {
        info!("{} sent a message to the server", username);
        let now = Local::now().format("%Y-%m-%d %H:%M:%S %z").to_string();
        let request = request_to_json!(
            "SendMessage",
            serde_json::json!({ "data": msg.trim(), "sender": username, "date": now })
        );
        sender.send((request, Some(client_addr))).unwrap();
    } else {
        let response = response_to_json!(400, "InvalidMessage");
        writer.write_all(response.as_bytes()).await.unwrap();
    }

    data.clear();
    Ok(())
}

// TODO: Fix the logic 
async fn validate_username<W: AsyncWrite + Unpin>(
    config: &Config,
    reader: &mut BufReader<ReadHalf<'_>>,
    writer: &mut W,
    client_addr: SocketAddr,
) -> Result<String> {
    loop {
        let mut request = String::new();
        let mut username = String::new();

        // TODO: Fix the issue - panic on client disconnection during the login procedure
        reader.read_line(&mut request).await.unwrap();
        // if (reader.read_line(&mut request).await).is_err() {
            // return Err(format!("{} disconnected before entering username", client_addr).into());
        // }
        // Need to get rid of unwrap and add the empty request/invalid request check
        let json_data: Value = serde_json::from_str(&request).unwrap();
        let (response, status_code) =
            if let Some("LogInUsername") = json_data.get("method").and_then(|v| v.as_str()) {
                username = json_data
                    .get("body")
                    .and_then(|v| v.as_str())
                    .unwrap()
                    .to_string();
                if config.is_valid_username(&username) {
                    (response_to_json!(200, "OK"), 200)
                } else {
                    (response_to_json!(400, "InvalidUsername"), 400)
                }
            } else {
                (response_to_json!(400, "BadRequest"), 400)
            };
        // TODO: Add send_data function to avoid code duplication
        if (writer.write_all(response.as_bytes()).await).is_err() {
            return Err(format!("{} disconnected before entering username", client_addr).into());
        }
        if status_code == 200 {
            return Ok(username);
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
    let request = request_to_json!(
        "Connection",
        serde_json::json!({ "data": disc_message, "date": now })
    );
    sender.send((request, Some(client_addr))).unwrap();
}
