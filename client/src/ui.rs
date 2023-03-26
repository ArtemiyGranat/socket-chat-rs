use crossterm::event::{Event, EventStream, KeyCode};
use futures::{FutureExt, StreamExt};
use std::io;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::mpsc,
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

enum InputMode {
    Normal,
    Insert,
}

pub struct App {
    input: String,
    input_mode: InputMode,
    messages: Vec<String>,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_mode: InputMode::Insert,
            messages: Vec::new(),
        }
    }
}

pub async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    socket: &mut TcpStream,
) -> io::Result<()> {
    let mut event_reader = EventStream::new();
    let (stream_reader, mut writer) = socket.split();

    let mut stream_reader = BufReader::new(stream_reader);
    let mut stdin_reader = BufReader::new(tokio::io::stdin());

    // let _username = validate_username(&mut stdin_reader, &mut stream_reader, &mut writer).await;

    let (tx, mut rx) = mpsc::channel::<String>(1);
    let mut line = String::new();
    loop {
        terminal.draw(|f| ui(f, &app))?;
        let event = event_reader.next().fuse();
        tokio::select! {
            _ = stream_reader.read_line(&mut line) => {
                app.messages.push(line.to_string());
                line.clear();
            }
            result = rx.recv() => {
                let msg = result.unwrap();
                writer
                    .write_all(&format!("{}\n", msg.trim()).as_bytes())
                    .await
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
                        InputMode::Insert => match key.code {
                            KeyCode::Enter => {
                                match tx.send(app.input.clone()).await {
                                    Ok(_) => (),
                                    Err(_) => {
                                        app.messages.push("Fail".to_string());
                                    }
                                }
                                app.messages.push(app.input.drain(..).collect());
                            }
                            KeyCode::Char(c) => {
                                app.input.push(c);
                            }
                            KeyCode::Backspace => {
                                app.input.pop();
                            }
                            KeyCode::Esc => {
                                app.input_mode = InputMode::Normal;
                            }
                            _ => {}
                        },
                    }
                }
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
        .split(f.size());

    let (msg, _) = match app.input_mode {
        InputMode::Normal => (
            vec![
                Span::raw(" Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("i", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to enter the insert mode"),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Insert => (
            vec![
                Span::raw(" Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to enter the normal mode, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to send a message"),
            ],
            Style::default(),
        ),
    };
    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let content = vec![Spans::from(Span::raw(format!("{}: {}", i, m)))];
            ListItem::new(content)
        })
        .collect();
    let messages = List::new(messages).block(Block::default().borders(Borders::ALL).title(msg));
    f.render_widget(messages, chunks[0]);

    let input = Paragraph::new(app.input.as_ref())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Insert => Style::default().fg(Color::Yellow),
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Enter the message"),
        );
    f.render_widget(input, chunks[1]);
    match app.input_mode {
        InputMode::Normal => {}
        InputMode::Insert => {
            f.set_cursor(chunks[1].x + app.input.width() as u16 + 1, chunks[1].y + 1)
        }
    }
}
