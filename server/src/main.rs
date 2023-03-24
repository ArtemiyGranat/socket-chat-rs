mod server;
use server::*;



#[tokio::main]
async fn main() {
    let server = Server::new().await;
    server.run_server().await;
}
