use ratatui::text::Line;

use super::constants::DEFAULT_STYLE;

pub fn create_headers<'a>(
    headers: impl IntoIterator<Item = &'a str>,
) -> impl Iterator<Item = Line<'a>> {
    headers
        .into_iter()
        .map(|h| Line::from(h).style(DEFAULT_STYLE).centered())
}
