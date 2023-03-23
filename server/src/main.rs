use chrono::Local;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{
        tcp::{ReadHalf, WriteHalf},
        TcpListener,
    },
    sync::broadcast,
};

mod config;
use config::*;

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

            let username = validate_username(&mut reader, &mut writer).await;
            println!(
                "[{}] [CONNECTION] {} ({:?}) has been connected to the server",
                Local::now(),
                username,
                addr
            );
            loop {
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        // TODO: Handle unwrap in match
                        // thread 'tokio-runtime-worker' panicked at 'called
                        // `Result::unwrap()` on an `Err` value: Custom { kind:
                        //  InvalidData, error: "stream did not contain valid
                        // UTF-8" }', src/main.rs:79:35
                        if result.unwrap() == 0 {
                            break;
                        }
                        print!("[{}] {}", Local::now(), line);
                        tx.send((line.clone(), addr)).unwrap();
                        line.clear();
                    }
                    result = rx.recv() => {
                        let (msg, sender_addr) = result.unwrap();
                        if addr != sender_addr {
                            writer.write_all(&msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
