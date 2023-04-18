use super::{
    chat::{MIN_HEIGHT, MIN_WIDTH},
    style::{bold, bold_colored, colored},
    util::{centered_rect, format_message},
    widgets::default_block,
};
use crate::{
    client::Client,
    message::Message,
    model::{ClientState, InputMode, Stage::*},
};
use tui::{
    backend::Backend,
    style::{Color, Style},
    widgets::{Block, Clear, ListItem, Paragraph, Wrap},
    Frame,
};

pub(crate) fn input_block(client: &Client, needed_input: String) -> Paragraph {
    Paragraph::new(client.input.as_ref())
        .style(match client.input_mode {
            InputMode::Insert if client.error_handler.is_none() => {
                Style::default().fg(Color::Yellow)
            }
            _ => Style::default(),
        })
        .block(default_block(format!(" Enter the {}", needed_input)))
}

pub(crate) fn message_block(messages: &[Message]) -> Vec<ListItem> {
    messages.iter().map(format_message).collect()
}

pub(crate) fn error_block<B: Backend>(f: &mut Frame<B>, client: &mut Client) {
    let error_message = client.error_handler.as_ref().unwrap();
    let block = Paragraph::new(error_message.as_ref())
        .block(default_block(vec![
            bold_colored(" Error!", Color::Red),
            " Press ".into(),
            bold("q"),
            " to continue".into(),
        ]))
        .wrap(Wrap { trim: true });
    f.render_widget(block, centered_rect(35, 10, f.size()));
}

pub(crate) fn too_small<B: Backend>(f: &mut Frame<B>, w: u16, h: u16) {
    let text = vec![
        "Terminal size is too small:".into(),
        vec![
            "Width = ".into(),
            colored(format!("{}", w), Color::Green),
            ", height = ".into(),
            colored(format!("{}", h), Color::Green),
        ]
        .into(),
        "Needed for current user interface:".into(),
        vec![
            "Width = ".into(),
            colored(format!("{}", MIN_WIDTH), Color::Green),
            ", height = ".into(),
            colored(format!("{}", MIN_HEIGHT), Color::Green),
        ]
        .into(),
    ];
    let paragraph = Paragraph::new(text.clone())
        .block(Block::default())
        .alignment(tui::layout::Alignment::Center)
        .wrap(Wrap { trim: true });

    let area = centered_rect(80, 40, f.size());
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

pub(crate) fn username_block(client: &Client) -> Paragraph {
    let input = match client.client_state {
        ClientState::LoggingIn(Username) | ClientState::Registering(Username) => {
            client.input.as_ref()
        }
        _ => client.username.as_ref(),
    };
    Paragraph::new(input)
        .style(match client.client_state {
            ClientState::LoggingIn(Username) | ClientState::Registering(Username) => {
                match client.input_mode {
                    InputMode::Insert if client.error_handler.is_none() => {
                        Style::default().fg(Color::Yellow)
                    }
                    _ => Style::default(),
                }
            }
            _ => Style::default(),
        })
        .block(default_block(" Enter the username"))
}

pub(crate) fn password_block(client: &Client) -> Paragraph {
    let input = match client.client_state {
        ClientState::LoggingIn(Password) | ClientState::Registering(Password) => {
            client.input.as_ref()
        }
        _ => "",
    };
    Paragraph::new(input)
        .style(match client.client_state {
            ClientState::LoggingIn(Password) | ClientState::Registering(Password) => {
                match client.input_mode {
                    InputMode::Insert if client.error_handler.is_none() => {
                        Style::default().fg(Color::Yellow)
                    }
                    _ => Style::default(),
                }
            }
            _ => Style::default(),
        })
        .block(default_block(" Enter the password"))
}
