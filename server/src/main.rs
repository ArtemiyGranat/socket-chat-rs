mod error;
mod server;

use server::*;

#[tokio::main]
async fn main() {
    match Server::new().await {
        Ok(mut server) => {
            if let Err(e) = server.run_server().await {
                eprintln!("[ERROR] {}", e);
            }
        }
        Err(e) => {
            eprintln!("[ERROR] {}", e);
        }
    }
}
