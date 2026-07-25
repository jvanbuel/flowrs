use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::constants::AirflowStateColor;
use super::theme::theme;

/// Truncate `s` to at most `max_cols` terminal columns, appending `…` if clipped.
///
/// Measures display width rather than counting `char`s, so wide characters
/// (CJK, emoji) cannot overflow the cell they are rendered into.
pub fn truncate_cols(s: &str, max_cols: usize) -> String {
    if s.width() <= max_cols {
        return s.to_string();
    }
    if max_cols == 0 {
        return String::new();
    }
    // Reserve one column for the ellipsis.
    let budget = max_cols - 1;
    let mut out = String::new();
    let mut used = 0;
    for ch in s.chars() {
        let w = ch.width().unwrap_or(0);
        if used + w > budget {
            break;
        }
        out.push(ch);
        used += w;
    }
    out.push('…');
    out
}

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

#[cfg(test)]
mod tests {
    use super::truncate_cols;

    #[test]
    fn truncate_appends_ellipsis_when_clipped() {
        assert_eq!(truncate_cols("hello", 10), "hello");
        assert_eq!(truncate_cols("hello world", 5), "hell…");
        assert_eq!(truncate_cols("hello", 0), "");
    }

    #[test]
    fn truncate_measures_display_width_not_char_count() {
        // Each CJK glyph occupies two columns, so only two fit alongside the
        // ellipsis in a six-column cell. Counting chars would have kept five
        // and overflowed to ten columns.
        assert_eq!(truncate_cols("日本語テスト", 6), "日本…");
        // A string of wide chars that exactly fills the cell is left alone.
        assert_eq!(truncate_cols("日本語", 6), "日本語");
    }
}
