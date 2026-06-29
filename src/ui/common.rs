use ratatui::{
    style::Style,
    text::{Line, Span},
};

use super::constants::AirflowStateColor;
use super::theme::theme;

pub fn create_headers<'a>(
    headers: impl IntoIterator<Item = &'a str>,
) -> impl Iterator<Item = Line<'a>> {
    let default_style = theme().default_style;
    headers
        .into_iter()
        .map(move |h| Line::from(h).style(default_style).centered())
}

pub fn state_to_colored_square<'a>(color: AirflowStateColor) -> Span<'a> {
    Span::styled("■", Style::default().fg(color.into()))
}
