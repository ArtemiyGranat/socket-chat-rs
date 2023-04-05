use crate::request_to_json;
use crate::ui::draw_ui;
use chrono::{DateTime, FixedOffset, Local};
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

#[derive(Debug)]
enum Command {
    Exit,
    SendMessage(String),
    LogInUsername(String),
}

pub(crate) struct Client {
    pub username: String,
    pub client_state: ClientState,
    pub input: String,
    pub input_mode: InputMode,
    pub messages: Vec<Value>,
    pub error_handler: Option<String>,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            username: String::new(),
            client_state: ClientState::LoggingIn,
            input: String::new(),
            input_mode: InputMode::Insert,
            messages: Vec::new(),
            error_handler: None,
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
        let (sender, mut receiver) = mpsc::channel::<Command>(1);
        let mut data = String::new();

        loop {
            terminal.draw(|f| draw_ui(f, &mut self))?;
            tokio::select! {
                received_data_size = stream_reader.read_line(&mut data) => {
                    if let Ok(0) = received_data_size {
                        self.handle_server_shutdown();
                        break Ok(())
                    }
                    self.handle_received_data(&data);
                    data.clear();
                }
                command = receiver.recv() => {
                    match command.unwrap() {
                        Command::SendMessage(data) => {
                            let request = request_to_json!("SendMessage", data);
                            writer
                                .write_all(request.as_bytes())
                                .await
                                .unwrap();
                        },
                        Command::LogInUsername(username) => {
                            let request = request_to_json!("LogInUsername", username);
                            writer
                                .write_all(request.as_bytes())
                                .await
                                .unwrap();
                        }
                        Command::Exit => break Ok(()),
                    }
                }
                result = event_reader.next().fuse() => {
                    if let Ok(Event::Key(key)) = result.unwrap() {
                        self.handle_input_event(key, &sender).await;
                    }
                }
            }
        }
    }

    fn handle_server_shutdown(&mut self) {
        let now = Local::now().format("%d-%m-%Y %H:%M").to_string();
        self.messages.push(
            serde_json::json!({"type": "message", "sender": "SERVER", "data": SERVER_SHUTDOWN_MESSAGE, "date": now }),
        );
    }

    fn handle_received_data(&mut self, data: &str) {
        let json_data: Value = serde_json::from_str(data).unwrap();
        match json_data.get("type").and_then(|data| data.as_str()) {
            Some("response") => self.handle_response(json_data),
            Some("request_s2c") => {
                unimplemented!()
            }
            _ => unreachable!(),
        }
    }

    fn handle_message(&mut self, json_data: Value) {
        if let ClientState::LoggedIn = self.client_state {
            self.messages.push(json_data);
        }
    }

    fn handle_response(&mut self, json_data: Value) {
        if let ClientState::LoggingIn = self.client_state {
            match json_data.get("status_code").and_then(|code| code.as_i64()) {
                Some(200) => {
                    self.client_state = ClientState::LoggedIn;
                }
                Some(400) => {
                    self.error_handler = json_data.get("message").map(|msg| msg.to_string());
                }
                _ => unreachable!("Invalid data {:?}", json_data),
            }
        } else {
            match json_data.get("status_code").and_then(|code| code.as_i64()) {
                Some(200) => {
                    // TODO: Implement 'Delivered' icon
                }
                Some(400) => {
                    self.error_handler = json_data.get("message").map(|msg| msg.to_string());
                    self.messages.pop();
                }
                _ => unreachable!("Invalid data {:?}", json_data),
            }
        }
    }

    async fn handle_input_event(&mut self, key: KeyEvent, sender: &Sender<Command>) {
        if self.error_handler.is_some() {
            if let KeyCode::Char('q') = key.code {
                self.error_handler = None;
                self.input.clear();
            }
        } else {
            match self.input_mode {
                InputMode::Normal => {
                    self.handle_normal_mode(key, sender).await;
                }
                InputMode::Insert => {
                    self.handle_insert_mode(key, sender).await;
                }
            }
        }
    }

    async fn handle_normal_mode(&mut self, key: KeyEvent, sender: &Sender<Command>) {
        match key.code {
            KeyCode::Char('i') => {
                self.input_mode = InputMode::Insert;
            }
            KeyCode::Char('q') => {
                sender.send(Command::Exit).await.unwrap();
            }
            _ => {}
        }
    }

    async fn handle_insert_mode(&mut self, key: KeyEvent, sender: &Sender<Command>) {
        match key.code {
            KeyCode::Enter => {
                let command = if let ClientState::LoggedIn = self.client_state {
                    Command::SendMessage(self.input.clone())
                } else {
                    Command::LogInUsername(self.input.clone())
                };
                sender.send(command).await.unwrap();

                if let ClientState::LoggedIn = self.client_state {
                    let now = Local::now().format("%d-%m-%Y %H:%M").to_string();
                    self.messages.push(json!({ "type": "message", 
                                               "sender": self.username,
                                               "data": self.input, 
                                               "date": now }));
                } else {
                    self.username = self.input.clone();
                }
                self.input.clear()
            }
            KeyCode::Char(c) => {
                self.input.push(c);
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
    }
}

fn from_json_str(json_str: &str) -> Value {
    let mut json_data: Value = serde_json::from_str(json_str).unwrap();

    let utc_date: DateTime<FixedOffset> = DateTime::parse_from_str(
        json_data
            .get("date")
            .and_then(|data| data.as_str())
            .unwrap(),
        "%Y-%m-%d %H:%M:%S %z",
    )
    .unwrap();
    let local_date: DateTime<Local> = utc_date.into();

    json_data["date"] = json!(local_date.format("%d-%m-%Y %H:%M").to_string());
    json_data
}
