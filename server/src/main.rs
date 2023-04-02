mod error;
mod server;

use server::*;

#[tokio::main]
async fn main() -> Result<(), error::ServerError> {
    let mut server = Server::new().await?;
    server.run_server().await?;
    Ok(())
}
