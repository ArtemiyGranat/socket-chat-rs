mod client;
mod config;
mod db;
mod macros;
mod server;

use log::{error, LevelFilter};
use simple_logger::SimpleLogger;
use time::macros::format_description;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .with_timestamp_format(format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second]"
        ))
        .init()
        .unwrap();

    let _pool = match db::connect().await {
        Some(pool) => pool,
        None => {
            error!("Could not connect to the database, check your .env file");
            return Ok(());
        }
    };

    if let Err(e) = server::run_server().await {
        error!("{}", e);
    };
    Ok(())
}
