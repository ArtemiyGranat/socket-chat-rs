use tui::{
    text::Spans,
    widgets::{Block, BorderType, Borders},
};

pub(crate) fn default_block<'a, T>(title: T) -> Block<'a>
where
    T: Into<Spans<'a>>,
{
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title.into())
}
