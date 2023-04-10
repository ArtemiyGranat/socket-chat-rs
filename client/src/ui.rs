use crate::{
    client::{Client, ClientState, InputMode},
    message::Message,
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use unicode_width::UnicodeWidthStr;

const MIN_WIDTH: u16 = 80;
const MIN_HEIGHT: u16 = 24;

pub(crate) fn ui<B: Backend>(f: &mut Frame<B>, client: &mut Client) {
    let (w, h) = (f.size().width, f.size().height);
    if w < MIN_WIDTH || h < MIN_HEIGHT {
        too_small_screen(f, w, h);
    } else {
        if let ClientState::LoggingIn = client.client_state {
            log_screen(f, client);
        } else {
            chat_screen(f, client);
        }
        if client.error_handler.is_some() {
            error_block(f, client);
        }
    }
}

fn too_small_screen<B: Backend>(f: &mut Frame<B>, w: u16, h: u16) {
    let text = vec![
        Spans::from("Terminal size is too small:"),
        Spans::from(vec![
            Span::raw("Width = "),
            Span::styled(format!("{}", w), Style::default().fg(Color::Green)),
            Span::raw(", height = "),
            Span::styled(format!("{}", h), Style::default().fg(Color::Green)),
        ]),
        Spans::from("Needed for current user interface:"),
        Spans::from(vec![
            Span::raw("Width = "),
            Span::styled(format!("{}", MIN_WIDTH), Style::default().fg(Color::Green)),
            Span::raw(", height = "),
            Span::styled(format!("{}", MIN_HEIGHT), Style::default().fg(Color::Green)),
        ]),
    ];

    let paragraph = Paragraph::new(text.clone())
        .block(Block::default())
        .alignment(tui::layout::Alignment::Center)
        .wrap(Wrap { trim: true });
    let area = centered_rect(80, 40, f.size());
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

fn log_screen<B: Backend>(f: &mut Frame<B>, client: &mut Client) {
    let block = input_block(client);
    let area = centered_rect(50, 20, f.size());
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    set_cursor(f, client, area);
}

fn chat_screen<B: Backend>(f: &mut Frame<B>, client: &mut Client) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
        .split(f.size());

    let help_message = help_message(&client.input_mode);

    let messages = client.messages.clone();
    let messages = List::new(message_block(&messages)).block(
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

    let input = input_block(client);
    f.render_widget(input, chunks[1]);
    set_cursor(f, client, chunks[1]);
}

fn help_message(input_mode: &InputMode) -> Vec<Span> {
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

fn message_block(messages: &[Message]) -> Vec<ListItem> {
    messages.iter().map(format_message).collect()
}

fn input_block(client: &mut Client) -> Paragraph {
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

fn error_block<B: Backend>(f: &mut Frame<B>, client: &mut Client) {
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

fn set_cursor<B: Backend>(f: &mut Frame<B>, client: &mut Client, area: Rect) {
    if let InputMode::Insert = client.input_mode {
        if client.error_handler.is_none() {
            f.set_cursor(area.x + client.input.width() as u16 + 1, area.y + 1);
        }
    }
}

fn format_message(message: &Message) -> ListItem {
    let date = Span::styled(
        format!("[{}] ", message.date),
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Rgb(216, 222, 233)),
    );
    let sender = message.sender.clone();
    let sender = sender
        .map(|sender| {
            Span::styled(
                format!("[{}] ", sender),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Rgb(129, 161, 193)),
            )
        })
        .unwrap_or_else(|| Span::raw(""));
    let data = Span::styled(
        &message.data,
        Style::default().fg(Color::Rgb(216, 222, 233)),
    );
    ListItem::new(vec![Spans::from(vec![date, sender, data])])
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
