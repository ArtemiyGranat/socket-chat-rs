use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::{client::Client, model::ClientState, model::Stage::*, ui::block::*, ui::util::*};

pub(crate) fn menu_screen<B: Backend>(f: &mut Frame<B>, client: &mut Client) {
    let rect = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(
            [
                Constraint::Percentage(30),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(40),
            ]
            .as_ref(),
        )
        .split(Rect::new(
            rect.width / 4,
            rect.y,
            rect.width / 2,
            rect.height,
        ));
    let log_in = Paragraph::new("Log in")
        .alignment(tui::layout::Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(match client.client_state {
                    ClientState::LoggingIn(_) => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                }),
        );
    let sign_up = Paragraph::new("Sign up")
        .alignment(tui::layout::Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(match client.client_state {
                    ClientState::Registering(_) => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                }),
        );
    f.render_widget(Clear, chunks[0]);
    f.render_widget(log_in, chunks[1]);
    f.render_widget(sign_up, chunks[2]);
    f.render_widget(Clear, chunks[3]);
}

pub(crate) fn log_in_screen<B: Backend>(f: &mut Frame<B>, client: &mut Client) {
    let rect = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(
            [
                Constraint::Percentage(30),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(40),
            ]
            .as_ref(),
        )
        .split(Rect::new(
            rect.width / 4,
            rect.y,
            rect.width / 2,
            rect.height,
        ));
    let username_block = username_block(client);
    let password_block = password_block(client);
    f.render_widget(Clear, chunks[0]);
    f.render_widget(username_block, chunks[1]);
    f.render_widget(password_block, chunks[2]);
    f.render_widget(Clear, chunks[3]);
    let area = match client.client_state {
        ClientState::LoggingIn(Username) | ClientState::Registering(Username) => chunks[1],
        ClientState::LoggingIn(Password) | ClientState::Registering(Password) => chunks[2],
        _ => unreachable!(),
    };
    set_cursor(f, client, area);
}
