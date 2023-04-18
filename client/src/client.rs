use crate::{
    message::Message,
    model::{InputMode, Request, Stage::*, State, SERVER_SHUTDOWN_MESSAGE},
    request_to_json,
    ui::chat,
};
use chrono::Local;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent};
use futures::{FutureExt, SinkExt};
use serde_json::Value;
use std::io;
use tokio::sync::mpsc::UnboundedSender;
use tokio::{net::TcpStream, sync::mpsc};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};
use tui::{backend::Backend, Terminal};

pub(crate) struct Client {
    pub username: String,
    pub state: State,
    pub input: String,
    pub input_mode: InputMode,
    pub messages: Vec<Message>,
    pub error_handler: Option<String>,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            username: String::new(),
            state: State::LoggingIn(Choosing),
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
        stream: TcpStream,
    ) -> io::Result<()> {
        let mut event_reader = EventStream::new();
        let mut lines = Framed::new(stream, LinesCodec::new());
        let (tx, mut rx) = mpsc::unbounded_channel::<Request>();

        loop {
            terminal.draw(|f| chat::ui(f, &mut self))?;
            tokio::select! {
                Some(command) = rx.recv() => {
                    match command {
                        Request::SendMessage(data) => {
                            let request = request_to_json!("SendMessage", data);
                            self.send_request(&mut lines, &request).await.unwrap();
                        },
                        Request::LogInUsername(username) => {
                            let request = request_to_json!("LogInUsername", username);
                            self.send_request(&mut lines, &request).await.unwrap();
                        }
                        Request::LogInPassword(password) => {
                            let request = request_to_json!("LogInPassword", password);
                            self.send_request(&mut lines, &request).await.unwrap();
                        }
                        Request::Exit => break Ok(()),
                    }
                },
                request = lines.next() => match request {
                    Some(Ok(received_data)) => self.handle_received_data(&received_data),
                    Some(Err(e)) => {
                        self.error_handler = Some(format!("Invalid request: {e}"));
                    }
                    None => {
                        self.handle_server_shutdown();
                        break Ok(())
                    }
                },
                result = event_reader.next().fuse() => {
                    if let Ok(Event::Key(key)) = result.unwrap() {
                        self.handle_input_event(key, &tx).await;
                    }
                },
            }
        }
    }

    fn handle_server_shutdown(&mut self) {
        let now = Local::now().format("%d-%m-%Y %H:%M").to_string();
        // TODO: Dont close app after server shutdown
        self.messages
            .push(Message::new(SERVER_SHUTDOWN_MESSAGE.to_string(), None, now));
    }

    fn handle_received_data(&mut self, data: &str) {
        let json_data: Value = serde_json::from_str(data).unwrap();
        match json_data.get("type").and_then(|v| v.as_str()) {
            Some("response") => self.handle_response(json_data),
            Some("request_s2c") => self.handle_request(json_data),
            _ => unreachable!(),
        }
    }

    fn handle_request(&mut self, json_data: Value) {
        match json_data.get("method").and_then(|v| v.as_str()) {
            Some("SendMessage") | Some("Connection") => {
                if let State::LoggedIn = self.state {
                    let message = Message::from_json_value(json_data);
                    self.messages.push(message);
                }
            }
            Some("MessageRead") => unimplemented!(),
            _ => unreachable!("Invalid data {:?}", json_data),
        }
    }

    fn handle_response(&mut self, json_data: Value) {
        match json_data.get("status_code").and_then(|v| v.as_i64()) {
            Some(200) => {
                match self.state {
                    State::LoggingIn(Username) | State::Registering(Username) => {
                        if let State::LoggingIn(_) = self.state {
                            self.state = State::LoggingIn(Password);
                        } else {
                            self.state = State::Registering(Password);
                        }
                    }
                    State::LoggingIn(Password) | State::Registering(Password) => {
                        self.state = State::LoggedIn;
                    }
                    State::LoggedIn => {
                        // TODO: Implement 'Delivered' icon
                    }
                    _ => unreachable!(),
                }
            }
            Some(400) => {
                self.error_handler = json_data
                    .get("message")
                    .map(|msg| msg.as_str().unwrap().to_string());
                // TODO: Implement new logic - push message to self.messages only if OK received
                if let State::LoggedIn = self.state {
                    self.messages.pop();
                }
            }
            _ => panic!("Invalid data {:?}", json_data),
        }
    }

    async fn handle_input_event(&mut self, key: KeyEvent, tx: &UnboundedSender<Request>) {
        if self.error_handler.is_none() {
            match self.state {
                State::LoggingIn(Choosing) | State::Registering(Choosing) => {
                    self.handle_menu_events(key, tx).await
                }
                _ => match self.input_mode {
                    InputMode::Normal => self.handle_normal_mode(key, tx).await,
                    InputMode::Insert => self.handle_insert_mode(key, tx).await,
                },
            }
        } else if let KeyCode::Char('q') = key.code {
            self.error_handler = None;
            self.input.clear();
        }
    }

    async fn handle_normal_mode(&mut self, key: KeyEvent, tx: &UnboundedSender<Request>) {
        match key.code {
            KeyCode::Char('i') => {
                self.input_mode = InputMode::Insert;
            }
            KeyCode::Char('q') => {
                tx.send(Request::Exit).unwrap();
            }
            _ => {}
        }
    }

    async fn handle_insert_mode(&mut self, key: KeyEvent, tx: &UnboundedSender<Request>) {
        match key.code {
            KeyCode::Enter => {
                let request = match self.state {
                    State::LoggedIn => Request::SendMessage(self.input.clone()),
                    State::LoggingIn(Username) | State::Registering(Username) => {
                        Request::LogInUsername(self.input.clone())
                    }
                    State::LoggingIn(Password) | State::Registering(Password) => {
                        Request::LogInPassword(self.input.clone())
                    }
                    _ => unreachable!(),
                };
                tx.send(request).unwrap();

                if let State::LoggedIn = self.state {
                    let now = Local::now().format("%d-%m-%Y %H:%M").to_string();
                    self.messages.push(Message::new(
                        self.input.clone(),
                        Some(self.username.clone()),
                        now,
                    ));
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

    async fn handle_menu_events(&mut self, key: KeyEvent, tx: &UnboundedSender<Request>) {
        match self.state {
            State::LoggingIn(Choosing) | State::Registering(Choosing) => match key.code {
                KeyCode::Down | KeyCode::Up | KeyCode::Char('j') | KeyCode::Char('k') => {
                    if let State::LoggingIn(_) = self.state {
                        self.state = State::Registering(Choosing);
                    } else {
                        self.state = State::LoggingIn(Choosing);
                    }
                }
                KeyCode::Enter => {
                    if let State::LoggingIn(_) = self.state {
                        self.state = State::LoggingIn(Username);
                    } else {
                        self.state = State::Registering(Username);
                    }
                }
                KeyCode::Char('q') => {
                    tx.send(Request::Exit).unwrap();
                }
                _ => {}
            },
            _ => {}
        }
    }

    async fn send_request(
        &self,
        lines: &mut Framed<TcpStream, LinesCodec>,
        request: &str,
    ) -> io::Result<()> {
        lines.send(request).await.unwrap();
        Ok(())
    }
}
