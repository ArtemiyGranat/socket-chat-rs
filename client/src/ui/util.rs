use super::style::{bold, bold_colored, colored};
use crate::{client::Client, message::Message, model::InputMode};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::Color,
    text::Span,
    widgets::ListItem,
    Frame,
};
use unicode_width::UnicodeWidthStr;

pub(crate) fn format_message(message: &Message) -> ListItem {
    let date = bold_colored(format!("[{}] ", message.date), Color::Rgb(216, 222, 233));
    let sender = message.sender.clone();
    let sender = sender
        .map(|sender| bold_colored(format!("[{}] ", sender), Color::Rgb(129, 161, 193)))
        .unwrap_or_else(|| "".into());
    let data = colored(&message.data, Color::Rgb(216, 222, 233));
    ListItem::new(vec![vec![date, sender, data].into()])
}

pub(crate) fn help_message(input_mode: &InputMode) -> Vec<Span> {
    match input_mode {
        InputMode::Normal => vec![
            " Press ".into(),
            bold("q"),
            " to exit, ".into(),
            bold("i"),
            " to enter the insert mode".into(),
        ],
        InputMode::Insert => vec![
            " Press ".into(),
            bold("Esc"),
            " to enter the normal mode, ".into(),
            bold("Enter"),
            " to send a message".into(),
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
