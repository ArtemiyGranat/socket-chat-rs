mod config;
mod error;
mod macros;
mod server;

use config::Config;
use log::{error, LevelFilter};
use server::*;
use simple_logger::SimpleLogger;
use time::macros::format_description;

#[tokio::main]
async fn main() -> Result<(), error::ServerError> {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .with_timestamp_format(format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second]"
        ))
        .init()
        .unwrap();
    let config = Config::default();
    if let Err(e) = run_server(&config).await {
        error!("{}", e.message);
    };
    Ok(())
}
