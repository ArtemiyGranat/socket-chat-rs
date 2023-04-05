mod config;
mod error;
mod macros;
mod server;

#[tokio::main]
async fn main() -> Result<(), error::ServerError> {
    let config = config::Config::default();
    server::run(&config).await?;
    Ok(())
}
