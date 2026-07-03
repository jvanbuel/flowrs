//! Pure text-measuring helpers for the trigger popup's param table.

/// Greedy word-wrap of `text` to `width` columns. Long words are hard-split.
pub(super) fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 || text.is_empty() {
        return Vec::new();
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let needs_space = !current.is_empty();
        let extra = usize::from(needs_space);
        if current.chars().count() + extra + word.chars().count() <= width {
            if needs_space {
                current.push(' ');
            }
            current.push_str(word);
            continue;
        }
        if !current.is_empty() {
            lines.push(std::mem::take(&mut current));
        }
        // Hard-split a word that is itself wider than the column.
        let mut chunk = String::new();
        for ch in word.chars() {
            if chunk.chars().count() == width {
                lines.push(std::mem::take(&mut chunk));
            }
            chunk.push(ch);
        }
        current = chunk;
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// Slide a `width`-column window over `value` so the char at `cursor_pos`
/// (a byte index) stays visible. Returns the text before the cursor, the
/// cursor char (a space if the cursor is at the end), and the text after.
pub(super) fn value_window(value: &str, cursor_pos: usize, width: usize) -> (String, char, String) {
    let cursor_pos = cursor_pos.min(value.len());
    let chars: Vec<char> = value.chars().collect();
    let cursor_col = value[..cursor_pos].chars().count();

    let width = width.max(1);
    let start = if cursor_col < width {
        0
    } else {
        cursor_col - width + 1
    };

    let before: String = chars[start..cursor_col].iter().collect();
    let cursor_char = chars.get(cursor_col).copied().unwrap_or(' ');
    let after_budget = width.saturating_sub(cursor_col - start + 1);
    let after_end = (cursor_col + 1 + after_budget).min(chars.len());
    let after: String = chars
        .get(cursor_col + 1..after_end)
        .map(|s| s.iter().collect())
        .unwrap_or_default();

    (before, cursor_char, after)
}

/// Truncate a string to at most `max_cols` columns, appending `…` if clipped.
pub(super) fn truncate_cols(s: &str, max_cols: usize) -> String {
    if s.chars().count() <= max_cols {
        return s.to_string();
    }
    if max_cols == 0 {
        return String::new();
    }
    let kept: String = s.chars().take(max_cols - 1).collect();
    format!("{kept}…")
}

#[cfg(test)]
mod tests {
    use super::{truncate_cols, value_window, wrap_text};

    #[test]
    fn wrap_text_breaks_on_word_boundaries() {
        assert_eq!(wrap_text("hello world foo", 11), vec!["hello world", "foo"]);
        assert!(wrap_text("anything", 0).is_empty());
        assert!(wrap_text("", 10).is_empty());
        // A word wider than the column is hard-split.
        assert_eq!(wrap_text("abcdefgh", 3), vec!["abc", "def", "gh"]);
    }

    #[test]
    fn truncate_appends_ellipsis_when_clipped() {
        assert_eq!(truncate_cols("hello", 10), "hello");
        assert_eq!(truncate_cols("hello world", 5), "hell…");
        assert_eq!(truncate_cols("hello", 0), "");
    }

    #[test]
    fn window_shows_whole_value_when_it_fits() {
        let (before, cursor, after) = value_window("abc", 1, 20);
        assert_eq!(before, "a");
        assert_eq!(cursor, 'b');
        assert_eq!(after, "c");
    }

    #[test]
    fn window_follows_cursor_past_the_right_edge() {
        // Cursor at end of a 10-char value in a 4-wide window: the tail must
        // be visible, and the cursor (a trailing space) sits at the edge.
        let value = "0123456789";
        let (before, cursor, after) = value_window(value, value.len(), 4);
        assert_eq!(cursor, ' ');
        assert_eq!(after, "");
        // before holds the last (width - 1) chars before the end.
        assert_eq!(before, "789");
    }

    #[test]
    fn window_total_never_exceeds_width() {
        let value = "0123456789";
        let (before, _cursor, after) = value_window(value, 5, 4);
        assert!(before.chars().count() + 1 + after.chars().count() <= 4);
    }
}
