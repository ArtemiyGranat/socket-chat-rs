use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::{client::Client, model::ClientState, ui::block::*, ui::util::*};

pub(crate) fn log_screen<B: Backend>(f: &mut Frame<B>, client: &mut Client) {
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
                    ClientState::LoggingIn => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                }),
        )
        .wrap(Wrap { trim: true });
    let sign_up = Paragraph::new("Sign up")
        .alignment(tui::layout::Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(match client.client_state {
                    ClientState::Registering => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                }),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(Clear, chunks[0]);
    f.render_widget(log_in, chunks[1]);
    f.render_widget(sign_up, chunks[2]);
    f.render_widget(Clear, chunks[3]);
}

pub(crate) fn log_in_screen<B: Backend>(f: &mut Frame<B>, client: &mut Client) {
    let block = input_block(client);
    let area = centered_rect(50, 20, f.size());
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    set_cursor(f, client, area);
}
