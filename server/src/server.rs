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
// const SHUTDOWN_MESSAGE: &str = "[SERVER] Server is shutting down\n";

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

    // TODO: On server shutdown send messages to all clients that server is
    // shutting down
    pub async fn run_server(&mut self) {
        let (tx, _) = broadcast::channel(MAX_CONNECTIONS);
        loop {
            let (mut client_socket, _) = self.listener.accept().await.unwrap();

            let tx = tx.clone();
            let mut rx = tx.subscribe();

            // Explanation: Code that handles the SIGINT signal and sends a 
            // message about it to all clients
            // let mut sigint = signal(SignalKind::interrupt()).unwrap();
            // let tx_shutdown = tx.clone();
            // tokio::spawn(async move {
            //     sigint.recv().await;
            //     tx_shutdown
            //         .send((SHUTDOWN_MESSAGE.to_string(), None))
            //         .unwrap();
            // });

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
                                    tx.send((disc_msg.clone(), Some(client_addr))).unwrap();
                                    break;
                                }
                                Ok(_) => {
                                    print!("[{}] {}", Local::now(), line);
                                    tx.send((line.clone(), Some(client_addr))).unwrap();
                                    line.clear();
                                }
                                Err(_) => {
                                    eprintln!("[ERROR] Stream did not contain valid UTF-8 data");
                                    break;
                                }
                            }
                        }
                        result = rx.recv() => {
                            let (msg, sender_addr) = result.unwrap();
                            // if msg == SHUTDOWN_MESSAGE {
                            //     writer.write_all(&msg.as_bytes()).await.unwrap();
                            //     std::process::exit(0)
                            // }
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
