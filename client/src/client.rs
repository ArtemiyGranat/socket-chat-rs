use tokio::{
    io::{Stdin, BufReader, AsyncBufReadExt, AsyncWriteExt},
    net::tcp::{ReadHalf, WriteHalf},
};

// TODO: Change return type to Result<String, std::io::Error>
pub async fn validate_username(
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
