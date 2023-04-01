use crate::error::ServerError;
use chrono::{Local, Utc};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{
        tcp::{ReadHalf, WriteHalf},
        TcpListener,
    },
    sync::broadcast,
};

const MAX_CONNECTIONS: usize = 10;
const SERVER_ADDRESS: &str = "localhost:8080";
const CONNECTION_MESSAGE: &str = "User has been connected to the server\n";
const DISCONNECTION_MESSAGE: &str = "User has been disconnected from the server\n";
// const SERVER_USERNAME: &str = "SERVER";

// TODO: Implement another macro for a system messages?
macro_rules! print_message {
    ($username:expr, $data:expr) => {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S");
        print!("[{}] [{}] {}", now, $username, $data)
    };
}

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub async fn new() -> Result<Self, ServerError> {
        match TcpListener::bind(SERVER_ADDRESS).await {
            Ok(listener) => Ok(Self { listener }),
            Err(_) => Err(ServerError {
                message: "Could not bind the server to this address".to_string(),
            }),
        }
    }

    pub async fn run_server(&mut self) -> Result<(), ServerError> {
        let (sender, _) = broadcast::channel(MAX_CONNECTIONS);
        loop {
            let (mut client_socket, _) = self.listener.accept().await.unwrap();
            let sender = sender.clone();
            let mut receiver = sender.subscribe();

            tokio::spawn(async move {
                // TODO: Store only client_socket and get addr by a function
                let client_addr = client_socket.peer_addr().unwrap();

                let (reader, mut writer) = client_socket.split();
                let mut reader = BufReader::new(reader);

                let username = match validate_username(&mut reader, &mut writer).await {
                    Ok(username) => {
                        print_message!(&username, CONNECTION_MESSAGE);
                        username
                    }
                    Err(e) => {
                        return Err(ServerError { message: e.message });
                    }
                };
                // TODO: Send connection message to all other clients.

                // TODO: Fix the issue
                // between the moment when the client has connected and has
                // not yet entered a nickname, all messages on the server are
                // stored and transmitted to the client after entering the nickname
                let mut data = String::new();

                loop {
                    tokio::select! {
                        received_data_size = reader.read_line(&mut data) => {
                            match received_data_size {
                                Ok(0) => {
                                    print_message!(&username, DISCONNECTION_MESSAGE);
                                    let json_data = to_json_string(username, DISCONNECTION_MESSAGE.to_string());
                                    sender.send((format!("{}\n", json_data), Some(client_addr))).unwrap();
                                    break Ok(());
                                }
                                Ok(_) => {
                                    print_message!(username, data);
                                    let json_data = to_json_string(username.clone(), data.clone());
                                    sender.send((format!("{}\n", json_data), Some(client_addr))).unwrap();
                                    data.clear();
                                }
                                // TODO: Add the lost connection error handling (look for ConnectionResetError)
                                Err(_) => {
                                    break Err(ServerError {
                                        message: "Stream did not contain valid UTF-8 data".to_string()
                                    });
                                },
                            }
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
            });
        }
    }
}

//  TODO: Check for the other error cases
async fn validate_username(
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

        let response = if username.is_empty() { "Error" } else { "Ok" };
        if (writer.write_all(format!("{}\n", response_to_json_string(response)).as_bytes()).await).is_err() {
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
    serde_json::json!({ "type": "message", "sender": username, "data": data, "date": now }).to_string()
}

fn response_to_json_string(response: &str) -> String {
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S %z").to_string();
    serde_json::json!({ "type": "response", "data": response, "date": now }).to_string()
}