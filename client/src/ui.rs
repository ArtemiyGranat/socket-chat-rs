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
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
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
    logged_in: bool,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_mode: InputMode::Insert,
            messages: Vec::new(),
            logged_in: false,
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
    let (tx, mut rx) = mpsc::channel::<String>(1);
    let mut data = String::new();

    loop {
        terminal.draw(|f| ui(f, &app))?;
        let event = event_reader.next().fuse();
        tokio::select! {
            received_data = stream_reader.read_line(&mut data) => {
                if received_data.unwrap() == 0 {
                    // TODO: implement this feature, working but not like i want it to
                    app.messages.push("[SERVER] Server is shutting down, app will be closed in 10 seconds".to_string());
                    return Ok(());
                }
                if !app.logged_in {
                    if data.trim() == "Ok" {
                        app.logged_in = true;
                    } else {
                        todo!("Need to implement error displaying");
                    }
                } else {
                    app.messages.push(data.to_string());
                }
                data.clear();
            }
            result = rx.recv() => {
                let msg = result.unwrap();
                writer
                    .write_all(&format!("{}\n", msg.trim()).as_bytes())
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
                        InputMode::Insert => match key.code {
                            KeyCode::Enter => {
                                match tx.send(app.input.clone()).await {
                                    Ok(_) => (),
                                    // TODO: Change this
                                    Err(_) => {
                                        app.messages.push("Fail".to_string());
                                    }
                                }
                                if app.logged_in {
                                    app.messages.push(app.input.drain(..).collect());
                                }
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
    if app.logged_in {
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
    } else {
        let block = Paragraph::new(app.input.as_ref())
            .style(match app.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Insert => Style::default().fg(Color::Yellow),
            })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Enter the username"),
            );
        let area = centered_rect(40, 10, f.size());
        f.render_widget(Clear, area); //this clears out the background
        f.render_widget(block, area);
        match app.input_mode {
            InputMode::Normal => {}
            InputMode::Insert => f.set_cursor(area.x + app.input.width() as u16 + 1, area.y + 1),
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
