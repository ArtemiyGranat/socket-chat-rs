mod server;
use server::*;

#[tokio::main]
async fn main() {
    let mut server = Server::new().await;
    server.run_server().await;
}
