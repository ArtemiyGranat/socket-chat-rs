use std::io::Write;

use chrono::Local;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{
        tcp::{ReadHalf, WriteHalf},
        TcpListener,
    },
    // signal::unix::{signal, SignalKind},
    sync::broadcast,
};

const MAX_CONNECTIONS: usize = 10;
const SERVER_ADDRESS: &str = "localhost:8080";
const CONNECTION_MESSAGE: &str = "[NEW CONNECTION] user has been connected to the server\n";
const DISCONNECTION_MESSAGE: &str = "[DISCONNECTION] user has been disconnected from the server\n";

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
            println!("NEW CONNECTION");
            let sender = sender.clone();
            let mut receiver = sender.subscribe();

            tokio::spawn(async move {
                // TODO: Store only client_socket and get addr by a function
                let client_addr = client_socket.peer_addr().unwrap();

                let (reader, mut writer) = client_socket.split();
                let mut reader = BufReader::new(reader);
                let mut line = String::new();
                let username = validate_username(&mut reader, &mut writer).await;

                print!(
                    "[{}] {}",
                    Local::now(),
                    CONNECTION_MESSAGE.replace("user", &username.clone())
                );
                // TODO: Send connection message to all other clients. It will
                // be implemented after authentification system will be implemented.

                // TODO: Fix the issue
                // between the moment when the client has connected and has
                // not yet entered a nickname, all messages on the server are
                // stored and transmitted to the client after entering the nickname
                loop {
                    tokio::select! {
                        result = reader.read_line(&mut line) => {
                            match result {
                                Ok(0) => {
                                    let disc_msg =
                                        DISCONNECTION_MESSAGE.replace("user", &username.clone());
                                    print!{"{}", disc_msg.clone()};
                                    sender.send((disc_msg.clone(), Some(client_addr))).unwrap();
                                    break;
                                }
                                Ok(_) => {
                                    print!("[{}] [{}] {}",
                                        Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                                        username,
                                        line);
                                    sender.send((format!(" {}", line), Some(client_addr))).unwrap();
                                    line.clear();
                                }
                                // TODO: Add the lost connection error handling
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
                                        writer.write_all(&msg.as_bytes()).await.unwrap();
                                    }
                                }
                                None => {
                                    writer.write_all(&msg.as_bytes()).await.unwrap();
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
