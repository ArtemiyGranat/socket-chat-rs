use tui::{backend::Backend, Frame, widgets::Clear};

use crate::{client::Client, ui::util::*, ui::block::*};

pub(crate) fn log_screen<B: Backend>(f: &mut Frame<B>, client: &mut Client) {
    let block = input_block(client);
    let area = centered_rect(50, 20, f.size());
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    set_cursor(f, client, area);
}