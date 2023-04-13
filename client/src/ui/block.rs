use super::{
    chat::{MIN_HEIGHT, MIN_WIDTH},
    util::{centered_rect, format_message},
};
use crate::{
    client::Client,
    message::Message,
    model::{ClientState, InputMode},
};
use tui::{
    backend::Backend,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Clear, ListItem, Paragraph, Wrap},
    Frame,
};

pub(crate) fn input_block(client: &mut Client) -> Paragraph {
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

pub(crate) fn message_block(messages: &[Message]) -> Vec<ListItem> {
    messages.iter().map(format_message).collect()
}

pub(crate) fn error_block<B: Backend>(f: &mut Frame<B>, client: &mut Client) {
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

pub(crate) fn too_small<B: Backend>(f: &mut Frame<B>, w: u16, h: u16) {
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
