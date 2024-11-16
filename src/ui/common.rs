use ratatui::{
    style::Style,
    text::{Line, Span},
};

use super::constants::{AirflowStateColor, DEFAULT_STYLE};

pub fn create_headers<'a>(
    headers: impl IntoIterator<Item = &'a str>,
) -> impl Iterator<Item = Line<'a>> {
    headers
        .into_iter()
        .map(|h| Line::from(h).style(DEFAULT_STYLE).centered())
}

pub fn state_to_colored_square<'a>(color: AirflowStateColor) -> Span<'a> {
    Span::styled("■", Style::default().fg(color.into()))
}
