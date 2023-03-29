use crate::client::ClientState;
use crate::client::{Client, InputMode};
use chrono::Local;
use crossterm::event::{KeyCode, KeyEvent};
use serde_json::{json, Value};
use tokio::sync::mpsc::Sender;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

fn _handle_normal_mode(app: &mut Client, key: KeyEvent) {
    match key.code {
        KeyCode::Char('i') => {
            app.input_mode = InputMode::Insert;
        }
        KeyCode::Char('q') => {
            // TODO: How to return q option?
            // std::process::exit(0)
        }
        _ => {}
    }
}

pub(crate) async fn handle_insert_mode(app: &mut Client, key: KeyEvent, tx: &Sender<String>) {
    match key.code {
        KeyCode::Enter => {
            match tx.send(app.input.clone()).await {
                Ok(_) => (),
                // TODO: Change the error message
                Err(_) => {
                    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                    app.messages
                        .push(json!({"username": "SERVER", "data": "Fail", "date": now }));
                }
            }
            let message: String = app.input.drain(..).collect();
            if let ClientState::LoggedIn = app.client_state {
                let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                app.messages
                    .push(json!({"username": app.username, "data": message, "date": now }));
            } else {
                app.username = message;
            }
            app.input.clear()
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
    }
}

pub(crate) fn draw_ui<B: Backend>(f: &mut Frame<B>, app: &mut Client) {
    if let ClientState::LoggedIn = app.client_state {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
            .split(f.size());

        let help_message = generate_help_message(&app.input_mode);

        let messages = app.messages.clone();
        let messages =
            List::new(generate_messages(&messages)).block(Block::default().borders(Borders::ALL).title(help_message));

        let messages_limit = chunks[0].height - 2;
        if app.messages.len() > messages_limit as usize {
            app.messages.remove(0);
        }
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
        let area = centered_rect(50, 20, f.size());
        f.render_widget(Clear, area);
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

fn generate_help_message(input_mode: &InputMode) -> Vec<Span> {
    match input_mode {
        InputMode::Normal => vec![
            Span::raw(" Press "),
            Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to exit, "),
            Span::styled("i", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to enter the insert mode"),
        ],
        InputMode::Insert => vec![
            Span::raw(" Press "),
            Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to enter the normal mode, "),
            Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to send a message"),
        ],
    }
}

fn generate_messages(messages: &[Value]) -> Vec<ListItem> {
    messages
        .iter()
        .map(|json_data| {
            let json_data = json_data.clone();
            let date = json_data["date"].as_str().unwrap().to_string();
            let username = json_data["username"].as_str().unwrap().to_string();
            let data = json_data["data"].as_str().unwrap().to_string();
            let content = vec![Spans::from(vec![
                Span::styled(
                    format!("[{}] ", date),
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Rgb(216, 222, 233)),
                ),
                Span::styled(
                    format!("[{}] ", username),
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Rgb(129, 161, 193)),
                ),
                Span::styled(data, Style::default().fg(Color::Rgb(216, 222, 233))),
            ])];
            ListItem::new(content)
        })
        .collect()
}
