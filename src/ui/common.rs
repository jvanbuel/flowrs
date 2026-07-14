use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders},
};

use super::constants::AirflowStateColor;
use super::theme::theme;

/// Builds a modal popup `Block` with the shared popup chrome: rounded, fully
/// bordered, themed border and background, and a title padded with a single
/// space on each side (`" title "`) styled bold in `accent`.
///
/// Centralizing this keeps every popup's title color language and padding
/// consistent instead of each popup rolling its own `Block` and spacing.
pub fn titled_popup_block(title: &str, accent: Color) -> Block<'static> {
    let t = theme();
    Block::default()
        .border_type(BorderType::Rounded)
        .borders(Borders::ALL)
        .border_style(t.border_style)
        .style(t.default_style)
        .title(format!(" {title} "))
        .title_style(Style::default().fg(accent).add_modifier(Modifier::BOLD))
}

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
