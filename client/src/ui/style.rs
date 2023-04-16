use std::borrow::Cow;

use tui::{
    style::{Color, Modifier, Style},
    text::Span,
};

pub fn bold<'a, T>(content: T) -> Span<'a>
where
    T: Into<Cow<'a, str>>,
{
    Span::styled(
        content.into(),
        Style::default().add_modifier(Modifier::BOLD),
    )
}

pub fn colored<'a, T>(content: T, color: Color) -> Span<'a>
where
    T: Into<Cow<'a, str>>,
{
    Span::styled(content.into(), Style::default().fg(color))
}

pub fn bold_colored<'a, T>(content: T, color: Color) -> Span<'a>
where
    T: Into<Cow<'a, str>>,
{
    Span::styled(
        content.into(),
        Style::default().add_modifier(Modifier::BOLD).fg(color),
    )
}
