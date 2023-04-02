use crate::config::Config;
use crate::error::ServerError;
use crate::{conn_message, disc_message};
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

// TODO: Implement another macro for a system messages?
macro_rules! print_message {
    ($username:expr, $data:expr) => {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S");
        print!("[{}] [{}] {}", now, $username, $data)
    };
}

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

    let mut _conn_message = String::new();
    let username = match validate_username(config, &mut reader, &mut writer).await {
        Ok(username) => {
            _conn_message = conn_message!(&username);
            username
        }
        Err(e) => {
            return Err(ServerError { message: e.message });
        }
    };
    // TODO: Send connection message to all other clients (_conn_message should be used).

    let mut data = String::new();
    loop {
        tokio::select! {
            received_data_size = reader.read_line(&mut data) => {
                if let Ok(0) = received_data_size {
                    let disc_message = disc_message!(&username);
                    sender
                        .send((
                            format!("{}\n", response_to_json_string(&disc_message)),
                            Some(client_addr),
                        ))
                        .unwrap();
                    break Ok(())
                }
                handle_received_data(
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
                match sender_addr {
                    Some(sender_addr) => {
                        if client_addr != sender_addr {
                            writer.write_all(message.as_bytes()).await.unwrap();
                        }
                    }
                    None => {
                        writer.write_all(message.as_bytes()).await.unwrap();
                    }
                }
            }
        }
    }
}

async fn handle_received_data(
    size: Result<usize, std::io::Error>,
    client_addr: SocketAddr,
    username: String,
    data: &mut String,
    sender: &mut Sender<(String, Option<SocketAddr>)>,
) -> Result<(), ServerError> {
    match size {
        Ok(_) => {
            print_message!(username, data);
            sender
                .send((
                    format!("{}\n", to_json_string(username, data.clone())),
                    Some(client_addr),
                ))
                .unwrap();
            data.clear();
        }
        // TODO: Add the lost connection error handling (look for ConnectionResetError)
        Err(e) => {
            return Err(ServerError {
                message: e.to_string(), // message: "Stream did not contain valid UTF-8 data".to_string()
            });
        }
    }
    Ok(())
}

//  TODO: Check for the other error cases
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
            len if len >= config.min_message_len && len <= config.max_message_len => "Ok",
            _ => "Error",
        };
        if (writer
            .write_all(format!("{}\n", response_to_json_string(response)).as_bytes())
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

fn to_json_string(username: String, data: String) -> String {
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S %z").to_string();
    serde_json::json!({ "type": "message", "sender": username, "data": data, "date": now })
        .to_string()
}

fn response_to_json_string(response: &str) -> String {
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S %z").to_string();
    serde_json::json!({ "type": "response", "data": response, "date": now }).to_string()
}
