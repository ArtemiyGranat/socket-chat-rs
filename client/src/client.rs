use crate::ui::{draw_ui, handle_insert_mode};
use chrono::{DateTime, Local, FixedOffset};
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent};
use futures::{FutureExt, StreamExt};
use serde_json::{json, Value};
use std::io;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::mpsc::{self, Sender},
};
use tui::{backend::Backend, Terminal};

const SERVER_SHUTDOWN_MESSAGE: &str = "Server is shutting down, app will be closed in 10 seconds";

#[derive(Clone, Copy)]
pub(crate) enum ClientState {
    LoggingIn,
    LoggedIn,
}

#[derive(Clone, Copy)]
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

impl Client {
    pub(crate) async fn run_client<B: Backend>(
        mut self,
        terminal: &mut Terminal<B>,
        socket: &mut TcpStream,
    ) -> io::Result<()> {
        let mut event_reader = EventStream::new();
        let (stream_reader, mut writer) = socket.split();
        let mut stream_reader = BufReader::new(stream_reader);
        let (sender, mut receiver) = mpsc::channel::<String>(1);
        let mut data = String::new();

        loop {
            terminal.draw(|f| draw_ui(f, &mut self))?;
            tokio::select! {
                outgoing_data_size = stream_reader.read_line(&mut data) => {
                    if let Ok(0) = outgoing_data_size {
                        self.handle_server_shutdown();
                        return Ok(());
                    }
                    self.handle_outgoing_data(&data);
                    data.clear();
                }
                result = receiver.recv() => {
                    let data = result.unwrap();
                    writer
                        .write_all(format!("{}\n", data.trim()).as_bytes())
                        .await
                        // TODO: Change this
                        .expect("Failed");
                    }
                result = event_reader.next().fuse() => {
                    if let Ok(Event::Key(key)) = result.unwrap() {
                        // TODO: Find a way to exit a client inside handle_input_event method
                        match self.input_mode {
                            InputMode::Normal => match key.code {
                                KeyCode::Char('i') => {
                                    self.input_mode = InputMode::Insert;
                                }
                                KeyCode::Char('q') => {
                                    return Ok(());
                                }
                                _ => {}
                            },
                            InputMode::Insert => {
                                handle_insert_mode(&mut self, key, &sender).await;
                            }
                        }
                    }
                }
            }
        }
    }

    fn handle_server_shutdown(&mut self) {
        let now = Local::now().format("%d-%m-%Y %H:%M").to_string();
        self.messages.push(serde_json::json!({"username": "SERVER", "data": SERVER_SHUTDOWN_MESSAGE, "date": now }));
    }

    fn handle_outgoing_data(&mut self, data: &str) {
        match self.client_state {
            ClientState::LoggedIn => {
                self.messages.push(from_json_string(data));
            }
            ClientState::LoggingIn if data.trim() == "Ok" => {
                self.client_state = ClientState::LoggedIn;
            }
            _ => {
                // if data.trim() == "Error"
                self.username.clear();
                todo!("Need to implement error displaying");
            }
        }
    }

    async fn _handle_input_event(&mut self, key: KeyEvent, sender: &Sender<String>) {
        match self.input_mode {
            InputMode::Normal => match key.code {
                KeyCode::Char('i') => {
                    self.input_mode = InputMode::Insert;
                }
                KeyCode::Char('q') => {
                    // TODO: How to exit a client from this method?
                }
                _ => {}
            },
            InputMode::Insert => {
                handle_insert_mode(self, key, sender).await;
            }
        }
    }
}


// TODO: Handle server messages
fn from_json_string(json_string: &str) -> Value {
    let mut json_data: Value = serde_json::from_str(json_string).unwrap();
    let utc_date: DateTime<FixedOffset> =
        DateTime::parse_from_str(json_data["date"].as_str().unwrap(), "%Y-%m-%d %H:%M:%S %z")
            .unwrap();
    let local_date: DateTime<Local> = utc_date.into();
    
    json_data["date"] = json!(local_date.format("%d-%m-%Y %H:%M").to_string());
    json_data
}
