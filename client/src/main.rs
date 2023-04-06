use crate::client::Client;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tokio::net::TcpStream;
use tui::{backend::CrosstermBackend, Terminal};

mod client;
mod macros;
mod message;
mod ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut socket = match TcpStream::connect("0.0.0.0:8080").await {
        Ok(socket) => socket,
        Err(_) => {
            eprintln!("[ERROR] Server is offline. Try again later");
            return Ok(());
        }
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let client = Client::default();
    if let Err(e) = client.run_client(&mut terminal, &mut socket).await {
        eprintln!("[ERROR] {}", e);
    }

    // TODO: Handle panics
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
