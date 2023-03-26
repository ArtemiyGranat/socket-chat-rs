use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io,
    sync::mpsc::{self, TryRecvError},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Stdin},
    net::{
        tcp::{ReadHalf, WriteHalf},
        TcpStream,
    },
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

use crate::client::*;

enum InputMode {
    Normal,
    Insert,
}

struct App {
    input: String,
    input_mode: InputMode,
    messages: Vec<String>,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
        }
    }
}

pub async fn run_tui() -> Result<(), Box<dyn Error>> {
    let mut socket = match TcpStream::connect("localhost:8080").await {
        Ok(socket) => socket,
        Err(_) => std::process::exit(1),
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::default();
    let res = run_app(&mut terminal, app, &mut socket).await;

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

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    socket: &mut TcpStream,
) -> io::Result<()> {
    // let (stream_reader, mut writer) = socket.split();

    // let mut stream_reader = BufReader::new(stream_reader);
    // let mut stdin_reader = BufReader::new(tokio::io::stdin());

    // let _username = validate_username(&mut stdin_reader, &mut stream_reader, &mut writer).await;

    let (tx, rx) = mpsc::channel::<String>();
    let mut stream_line = String::new();
    let mut stdin_line = String::new();
    loop {
        terminal.draw(|f| ui(f, &app))?;

        match rx.try_recv() {
            Ok(msg) => {
                socket
                    .write_all(&format!("{}\n", msg.trim()).as_bytes())
                    .await
                    .expect("Failed");
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => std::process::exit(1),
        }

        if let Event::Key(key) = event::read().unwrap() {
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
                        match tx.send(app.input.clone()) {
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
