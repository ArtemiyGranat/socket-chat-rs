use crate::config::Config;
use crate::Result;
use crate::{conn_message, disc_message, message_to_json, response_to_json};
use chrono::{Local, Utc};
use log::info;
use std::net::SocketAddr;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{
        tcp::{ReadHalf, WriteHalf},
        TcpListener, TcpStream,
    },
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

    let username = validate_username(client_addr, config, &mut reader, &mut writer).await?;
    let _conn_message = conn_message!(&username);
    info!(
        "{} ({}) has been connected to the server",
        username, client_addr
    );
    // TODO: Send connection message to all other clients (_conn_message should be used).

    let mut data = String::new();
    loop {
        tokio::select! {
            received_data_size = reader.read_line(&mut data) => {
                if let Ok(0) = received_data_size {
                    info!("{} ({}) has been disconnected from the server", username, client_addr);
                    sender
                        .send((
                            response_to_json!(&disc_message!(&username)),
                            Some(client_addr),
                        ))
                        .unwrap();
                    break Ok(())
                }
                handle_received_data(
                    config,
                    received_data_size,
                    client_addr,
                    username.clone(),
                    &mut data,
                    sender,
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

async fn handle_received_data(
    config: &Config,
    size: std::io::Result<usize>,
    client_addr: SocketAddr,
    username: String,
    data: &mut String,
    sender: &mut Sender<(String, Option<SocketAddr>)>,
) -> Result<()> {
    if let Err(e) = size {
        return Err(e.into());
    }
    match data.trim().len() {
        len if len >= config.min_message_len && len <= config.max_message_len => {
            info!("{} sent a message to the server", username);
            sender
                .send((message_to_json!(username, data.clone()), Some(client_addr)))
                .unwrap();
        }
        _ => {
            todo!();
        }
    }
    data.clear();
    Ok(())
}

async fn validate_username(
    client_addr: SocketAddr,
    config: &Config,
    reader: &mut BufReader<ReadHalf<'_>>,
    writer: &mut WriteHalf<'_>,
) -> Result<String> {
    let mut username = String::new();
    loop {
        if (reader.read_line(&mut username).await).is_err() {
            return Err(format!("{} disconnected before entering username", client_addr).into());
        }
        username = username.trim().to_string();
        let response = match username.len() {
            len if len >= config.min_username_len && len <= config.max_username_len => "Ok",
            _ => "Error",
        };
        if (writer
            .write_all(response_to_json!(response).as_bytes())
            .await)
            .is_err()
        {
            return Err(format!("{} disconnected before entering username", client_addr).into());
        }
        if let "Ok" = response {
            return Ok(username);
        }
        username.clear();
    }
}
