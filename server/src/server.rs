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
const SERVER_ADDRESS: &str = "0.0.0.0:8080";
const CONNECTION_MESSAGE: &str = "user has been connected to the server\n";
const DISCONNECTION_MESSAGE: &str = "user has been disconnected from the server\n";
const SERVER_USERNAME: &str = "SERVER";

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub async fn new() -> Self {
        let listener = match TcpListener::bind(SERVER_ADDRESS).await {
            Ok(listener) => listener,
            Err(_) => {
                eprintln!("[ERROR] Could not bind the server to this address");
                std::process::exit(1)
            }
        };
        Self { listener }
    }

    pub async fn run_server(&mut self) {
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
                let username = validate_username(&mut reader, &mut writer).await;

                print!(
                    "[{}] {}",
                    Local::now().format("%Y-%m-%d %H:%M:%S"),
                    CONNECTION_MESSAGE.replace("user", &username.clone())
                );
                // TODO: Send connection message to all other clients. It will
                // be implemented after authentification system will be implemented.

                // TODO: Fix the issue
                // between the moment when the client has connected and has
                // not yet entered a nickname, all messages on the server are
                // stored and transmitted to the client after entering the nickname
                let mut data = String::new();
                loop {
                    tokio::select! {
                        result = reader.read_line(&mut data) => {
                            let now = Local::now().format("%Y-%m-%d %H:%M:%S");
                            match result {
                                Ok(0) => {
                                    let disc_msg =
                                        DISCONNECTION_MESSAGE.replace("user", &username.clone());
                                    print!{"[{}] {}", now, disc_msg};
                                    let json_data = to_json_string(SERVER_USERNAME.to_string(), disc_msg);
                                    sender.send((format!("{}\n", json_data), Some(client_addr))).unwrap();
                                    break;
                                }
                                Ok(_) => {
                                    print!("[{}] [{}] {}",
                                        now,
                                        username,
                                        data);
                                    let json_data = to_json_string(username.clone(), data.clone());
                                    sender.send((format!("{}\n", json_data), Some(client_addr))).unwrap();
                                    data.clear();
                                }
                                // TODO: Add the lost connection error handling (look for ConnectionResetError)
                                Err(_) => {
                                    eprintln!("[ERROR] Stream did not contain valid UTF-8 data");
                                    break;
                                }
                            }
                        }
                        result = receiver.recv() => {
                            let (msg, sender_addr) = result.unwrap();
                            match sender_addr {
                                Some(sender_addr) => {
                                    if client_addr != sender_addr {
                                        writer.write_all(msg.as_bytes()).await.unwrap();
                                    }
                                }
                                None => {
                                    writer.write_all(msg.as_bytes()).await.unwrap();
                                }
                            }

                        }
                    }
                }
            });
        }
    }
}

// TODO: Fix the issue: reader.read_line...unwrap() -> match
// thread 'tokio-runtime-worker' panicked at 'called `Result::unwrap()` on an
// `Err` value: Os { code: 54, kind: ConnectionReset, message: "Connection reset
// by peer" }', src/server.rs:125:47
// TODO: Fix the issue
// thread 'tokio-runtime-worker' panicked at 'called `Result::unwrap()` on an
// `Err` value: Os { code: 32, kind: BrokenPipe, message: "Broken pipe" }',
// src/server.rs:133:14
async fn validate_username(
    reader: &mut BufReader<ReadHalf<'_>>,
    writer: &mut WriteHalf<'_>,
) -> String {
    let mut username = String::new();
    loop {
        reader.read_line(&mut username).await.unwrap();
        username = username.trim().to_string();

        let response = if username.is_empty() { "Error" } else { "Ok" };
        writer
            .write_all(format!("{}\n", response).as_bytes())
            .await
            .unwrap();
        if let "Error" = response {
            username.clear();
            continue;
        }
        return username;
    }
}

fn to_json_string(username: String, data: String) -> String {
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S %z").to_string();
    serde_json::json!({"username": username, "data": data, "date": now }).to_string()
}
