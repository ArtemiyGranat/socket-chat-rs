use chrono::Local;
use tokio::io::Stdin;
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

// TODO: Change return type to Result<String, std::io::Error>
async fn validate_username(
    stdin_reader: &mut BufReader<Stdin>,
    stream_reader: &mut BufReader<ReadHalf<'_>>,
    writer: &mut WriteHalf<'_>,
) -> String {
    let mut username = String::new();
    let mut response = String::new();
    loop {
        tokio::select! {
            received_data = stream_reader.read_line(&mut response) => {
                if received_data.unwrap() == 0 || response.trim() == "Ok" {
                    return username;
                }
                eprintln!("[ERROR] Invalid username. Try again");
                response.clear();
            }
            _ = stdin_reader.read_line(&mut username) => {
                writer.write_all(&username.as_bytes()).await.unwrap();
                username.clear();
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let mut socket = match TcpStream::connect("localhost:8080").await {
        Ok(socket) => socket,
        Err(_) => {
            eprintln!("[ERROR] Could not connect to this server. Try again later.");
            std::process::exit(1)
        }
    };

    let (stream_reader, mut writer) = socket.split();

    let mut stream_reader = BufReader::new(stream_reader);
    let mut stdin_reader = BufReader::new(tokio::io::stdin());

    let _username = validate_username(&mut stdin_reader, &mut stream_reader, &mut writer).await;
    let mut stream_line = String::new();
    let mut stdin_line = String::new();
    loop {
        tokio::select! {
            received_data = stream_reader.read_line(&mut stream_line) => {
                if received_data.unwrap() == 0 {
                    break;
                }
                print!("[{}] {}", Local::now(), stream_line);
                stream_line.clear();
            }
            _ = stdin_reader.read_line(&mut stdin_line) => {
                writer.write_all(&stdin_line.as_bytes()).await.unwrap();
                stdin_line.clear();
            }
        }
    }
}
