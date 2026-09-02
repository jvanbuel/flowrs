use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders},
};
use unicode_width::UnicodeWidthStr;

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
    // Reserve one column for the ellipsis. The prefix is measured as a string
    // after each append rather than by summing per-char widths, because a
    // sequence such as an emoji presentation selector can widen the preceding
    // char without carrying any width of its own.
    let budget = max_cols - 1;
    let mut out = String::new();
    for ch in s.chars() {
        out.push(ch);
        if out.as_str().width() > budget {
            out.pop();
            break;
        }
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
    use unicode_width::UnicodeWidthStr;

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

    #[test]
    fn truncate_bounds_grapheme_sequences_by_string_width() {
        // U+2639 is one column on its own, but the U+FE0F presentation selector
        // after it makes the pair render as a two-column emoji. Summing per-char
        // widths would have kept both and overflowed a two-column cell.
        let out = truncate_cols("\u{2639}\u{FE0F}x", 2);
        assert!(
            out.as_str().width() <= 2,
            "got {out:?} at width {}",
            out.as_str().width()
        );
        assert!(out.ends_with('…'));
    }
}
