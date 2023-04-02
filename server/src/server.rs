use crate::config::Config;
use crate::error::ServerError;
use crate::{conn_message, disc_message, message_to_json, print_message, response_to_json};
use chrono::{Local, Utc};
use std::net::SocketAddr;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{
        tcp::{ReadHalf, WriteHalf},
        TcpListener, TcpStream,
    },
    sync::broadcast::{self, Sender},
};

async fn init_server(config: &Config) -> Result<TcpListener, ServerError> {
    match TcpListener::bind(config.server_address.clone()).await {
        Ok(listener) => Ok(listener),
        Err(_) => Err(ServerError {
            message: "Could not bind the server to this address".to_string(),
        }),
    }
}

pub async fn run_server(config: &Config) -> Result<(), ServerError> {
    let listener = init_server(config).await?;
    let (sender, _) = broadcast::channel(config.max_connections);
    loop {
        let config = config.clone();
        let (client_socket, _) = listener.accept().await.unwrap();
        let mut sender = sender.clone();

        tokio::spawn(async move {
            handle_client(&config, client_socket, &mut sender)
                .await
                .unwrap();
        });
    }
}

async fn handle_client(
    config: &Config,
    mut client_socket: TcpStream,
    sender: &mut Sender<(String, Option<SocketAddr>)>,
) -> Result<(), ServerError> {
    let mut receiver = sender.subscribe();
    let client_addr = client_socket.peer_addr().unwrap();
    let (reader, mut writer) = client_socket.split();
    let mut reader = BufReader::new(reader);

    let username = validate_username(config, &mut reader, &mut writer).await?;
    let _conn_message = conn_message!(&username);
    // TODO: Send connection message to all other clients (_conn_message should be used).

    let mut data = String::new();
    loop {
        tokio::select! {
            received_data_size = reader.read_line(&mut data) => {
                if let Ok(0) = received_data_size {
                    let disc_message = disc_message!(&username);
                    sender
                        .send((
                            response_to_json!(&disc_message),
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
    size: Result<usize, std::io::Error>,
    client_addr: SocketAddr,
    username: String,
    data: &mut String,
    sender: &mut Sender<(String, Option<SocketAddr>)>,
) -> Result<(), ServerError> {
    if let Err(e) = size {
        return Err(ServerError {
            message: e.to_string(),
        });
    }
    match data.trim().len() {
        len if len >= config.min_message_len && len <= config.max_message_len => {
            print_message!(username, data);
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
    config: &Config,
    reader: &mut BufReader<ReadHalf<'_>>,
    writer: &mut WriteHalf<'_>,
) -> Result<String, ServerError> {
    let mut username = String::new();
    loop {
        if (reader.read_line(&mut username).await).is_err() {
            return Err(ServerError {
                message: "Client disconnected before entering username".to_string(),
            });
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
            return Err(ServerError {
                message: "Client disconnected before entering username".to_string(),
            });
        }
        if let "Ok" = response {
            return Ok(username);
        }
        username.clear();
    }
}
