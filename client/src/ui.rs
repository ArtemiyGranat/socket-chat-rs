use crate::client::{Client, ClientState, InputMode};
use serde_json::Value;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use unicode_width::UnicodeWidthStr;

pub(crate) fn draw_ui<B: Backend>(f: &mut Frame<B>, client: &mut Client) {
    if let ClientState::LoggedIn = client.client_state {
        draw_chat_screen(f, client);
    } else {
        draw_log_screen(f, client);
    }
    if client.error_handler.is_some() {
        display_error(f, client);
    }
}

fn draw_chat_screen<B: Backend>(f: &mut Frame<B>, client: &mut Client) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
        .split(f.size());

    let help_message = generate_help_message(&client.input_mode);

    let messages = client.messages.clone();
    let messages = List::new(generate_messages(&messages)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(help_message),
    );

    let messages_limit = chunks[0].height - 2;
    if client.messages.len() > messages_limit as usize {
        client.messages.remove(0);
    }
    f.render_widget(messages, chunks[0]);

    let input = draw_input_block(client);
    f.render_widget(input, chunks[1]);
    set_cursor(f, client, chunks[1]);
}

fn draw_log_screen<B: Backend>(f: &mut Frame<B>, client: &mut Client) {
    let block = draw_input_block(client);
    let area = centered_rect(50, 20, f.size());
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    set_cursor(f, client, area);
}

fn draw_input_block(client: &mut Client) -> Paragraph {
    let needed_input = if let ClientState::LoggedIn = client.client_state {
        "message"
    } else {
        "username"
    };
    Paragraph::new(client.input.as_ref())
        .style(match client.input_mode {
            InputMode::Insert if client.error_handler.is_none() => {
                Style::default().fg(Color::Yellow)
            }
            _ => Style::default(),
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(format!(" Enter the {}", needed_input)),
        )
}

fn set_cursor<B: Backend>(f: &mut Frame<B>, client: &mut Client, area: Rect) {
    if let InputMode::Insert = client.input_mode {
        if client.error_handler.is_none() {
            f.set_cursor(area.x + client.input.width() as u16 + 1, area.y + 1);
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
            let date = json_data
                .get("date")
                .and_then(|data| data.as_str())
                .unwrap();
            let data = json_data
                .get("data")
                .and_then(|data| data.as_str())
                .unwrap();
            let sender = json_data
                .get("sender")
                .and_then(|data| data.as_str())
                .unwrap();
            display_message(date, sender, data)
        })
        .collect()
}

fn display_message<'a>(date: &'a str, sender: &'a str, data: &'a str) -> ListItem<'a> {
    let content = vec![Spans::from(vec![
        Span::styled(
            format!("[{}] ", date),
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Rgb(216, 222, 233)),
        ),
        Span::styled(
            format!("[{}] ", sender),
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Rgb(129, 161, 193)),
        ),
        Span::styled(data, Style::default().fg(Color::Rgb(216, 222, 233))),
    ])];
    ListItem::new(content)
}

fn display_error<B: Backend>(f: &mut Frame<B>, client: &mut Client) {
    let error_message = client.error_handler.as_ref().unwrap();
    let block = Paragraph::new(error_message.as_ref())
        .block(
            Block::default()
                .title(vec![
                    Span::styled(
                        " Error!",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Press "),
                    Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to continue"),
                ])
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .wrap(Wrap { trim: true });
    let area = centered_rect(35, 10, f.size());
    f.render_widget(block, area);
}
