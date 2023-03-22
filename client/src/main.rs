use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let mut _stream = TcpStream::connect("localhost:8080").await.unwrap();
}
