mod config;
mod error;
mod server;

use server::*;
use config::Config;

#[tokio::main]
async fn main() -> Result<(), error::ServerError> {
    let server = Server::new().await?;
    // TODO: Need to decide which one to use: config from main or config from Self
    let config = Config::default();
    server.run_server(&config).await?;
    Ok(())
}
