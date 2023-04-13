use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Color},
    text::{Span, Spans},
    Frame, widgets::ListItem,
};
use unicode_width::UnicodeWidthStr;

use crate::{client::Client, model::InputMode, message::Message};

pub(crate) fn format_message(message: &Message) -> ListItem {
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

pub(crate) fn help_message(input_mode: &InputMode) -> Vec<Span> {
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

pub(crate) fn set_cursor<B: Backend>(f: &mut Frame<B>, client: &mut Client, area: Rect) {
    if let InputMode::Insert = client.input_mode {
        if client.error_handler.is_none() {
            f.set_cursor(area.x + client.input.width() as u16 + 1, area.y + 1);
        }
    }
}

pub(crate) fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
