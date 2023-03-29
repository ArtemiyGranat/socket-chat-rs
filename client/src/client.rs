use crate::ui::{draw_ui, handle_insert_mode};
use chrono::{DateTime, Local, Utc};
use crossterm::event::{Event, EventStream, KeyCode};
use futures::{FutureExt, StreamExt};
use serde_json::{json, Value};
use std::io;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::mpsc::{self},
};
use tui::{backend::Backend, Terminal};

const SERVER_SHUTDOWN_MESSAGE: &str = "Server is shutting down, app will be closed in 10 seconds";

#[derive(Clone, Copy)]
pub(crate) enum ClientState {
    LoggingIn,
    LoggedIn,
}

pub(crate) enum InputMode {
    Normal,
    Insert,
}

pub(crate) struct Client {
    pub username: String,
    pub client_state: ClientState,
    pub input: String,
    pub input_mode: InputMode,
    pub messages: Vec<Value>,
}

impl Default for Client {
    fn default() -> Client {
        Client {
            username: String::new(),
            client_state: ClientState::LoggingIn,
            input: String::new(),
            input_mode: InputMode::Insert,
            messages: Vec::new(),
        }
    }
}

pub(crate) async fn run_client<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: Client,
    socket: &mut TcpStream,
) -> io::Result<()> {
    let mut event_reader = EventStream::new();
    let (stream_reader, mut writer) = socket.split();
    let mut stream_reader = BufReader::new(stream_reader);
    let (tx, mut rx) = mpsc::channel::<String>(1);
    let mut data = String::new();

    loop {
        terminal.draw(|f| draw_ui(f, &mut app))?;
        let event = event_reader.next().fuse();
        tokio::select! {
            received_data_size = stream_reader.read_line(&mut data) => {
                if let Ok(0) = received_data_size {
                    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                    app.messages.push(serde_json::json!({"username": "SERVER", "data": SERVER_SHUTDOWN_MESSAGE, "date": now }));
                    return Ok(());
                }
                match app.client_state {
                    ClientState::LoggedIn => {
                        app.messages.push(from_json_string(&data));
                    }
                    ClientState::LoggingIn if data.trim() == "Ok" => {
                        app.client_state = ClientState::LoggedIn;
                    }
                    _ => {
                        // if data.trim() == "Error"
                        app.username.clear();
                        todo!("Need to implement error displaying");
                    }
                }
                data.clear();
            }
            result = rx.recv() => {
                let msg = result.unwrap();
                writer
                    .write_all(format!("{}\n", msg.trim()).as_bytes())
                    .await
                    // TODO: Change this
                    .expect("Failed");
                }
            result = event => {
                if let Ok(Event::Key(key)) = result.unwrap() {
                    match app.input_mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Char('i') => {
                                app.input_mode = InputMode::Insert;
                            }
                            KeyCode::Char('q') => {
                                return Ok(());
                            }
                            _ => {}
                        },
                        InputMode::Insert => { handle_insert_mode(&mut app, key, &tx).await; }
                    }
                }
            }
        }
    }
}

// TODO: Change variable names
// TODO: Handle server messages
fn from_json_string(json_string: &str) -> Value {
    let mut json_data: Value = serde_json::from_str(json_string).unwrap();
    let utc_date: DateTime<Utc> =
        DateTime::parse_from_str(json_data["date"].as_str().unwrap(), "%Y-%m-%d %H:%M:%S %z")
            .unwrap()
            .into();
    let local_date: DateTime<Local> = DateTime::from(utc_date);
    let now_str = local_date.format("%Y-%m-%d %H:%M:%S").to_string();
    json_data["date"] = json!(now_str);

    json_data
}
