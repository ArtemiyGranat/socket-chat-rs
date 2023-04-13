use super::{block::{error_block, input_block, message_block, too_small}, authorization::log_screen, util::{help_message, set_cursor}};
use crate::{
    client::Client,
    model::ClientState,
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, BorderType, Borders, List},
    Frame,
};

pub(crate) const MIN_WIDTH: u16 = 80;
pub(crate) const MIN_HEIGHT: u16 = 24;

pub(crate) fn ui<B: Backend>(f: &mut Frame<B>, client: &mut Client) {
    let (w, h) = (f.size().width, f.size().height);
    if w < MIN_WIDTH || h < MIN_HEIGHT {
        too_small(f, w, h);
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
