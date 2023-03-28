use crate::client::{run_client, Client};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tokio::net::TcpStream;
use tui::{backend::CrosstermBackend, Terminal};

mod client;
mod ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut socket = match TcpStream::connect("localhost:8080").await {
        Ok(socket) => socket,
        Err(_) => {
            eprintln!("[ERROR] Server is offline. Try again later");
            std::process::exit(1)
        }
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = Client::default();
    let res = run_client(&mut terminal, app, &mut socket).await;

    // TODO: Handle the errors and disable raw mode anyway
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}
