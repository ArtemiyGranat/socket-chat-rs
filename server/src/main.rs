use chrono::Local;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::broadcast,
};

mod config;
use config::*;

#[tokio::main]
async fn main() {
    let listener = match TcpListener::bind(SERVER_ADDRESS).await {
        Ok(listener) => listener,
        Err(_) => {
            eprintln!("[ERROR] Could not bind the server to this address");
            std::process::exit(1)
        }
    };

    let (tx, _) = broadcast::channel(MAX_CONNECTIONS);

    loop {
        let (mut client_socket, addr) = listener.accept().await.unwrap();
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            let (reader, mut writer) = client_socket.split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            loop {
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            break;
                        }
                        print!("[{}] {}", Local::now(), line);
                        tx.send((line.clone(), addr)).unwrap();
                        line.clear();
                    }
                    result = rx.recv() => {
                        let (msg, other_addr) = result.unwrap();
                        if addr != other_addr {
                            writer.write_all(&msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
