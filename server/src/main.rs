mod config;
mod error;
mod macros;
mod server;

use config::Config;
use server::*;

#[tokio::main]
async fn main() -> Result<(), error::ServerError> {
    let config = Config::default();
    run_server(&config).await?;
    Ok(())
}
