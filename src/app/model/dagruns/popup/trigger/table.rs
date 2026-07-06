//! Cell contents and column sizing for the trigger popup's param table.

use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span, Text},
};

use crate::ui::theme::theme;

use super::params::{ParamEntry, ParamKind};
use super::text::{truncate_cols, value_window, wrap_text};
use super::TriggerDagRunPopUp;

/// Total inter-column padding the table reserves (1 col between each of 3 columns).
const COLUMN_GAPS: usize = 2;
/// Max wrapped lines per cell (descriptions and enum values wrap).
const MAX_CELL_LINES: usize = 3;

impl TriggerDagRunPopUp {
    /// Compute the (key, value, description) column widths for a popup of
    /// `inner_w` columns.
    ///
    /// The key column is content-sized, capped at a quarter of the popup. The
    /// value column grows with its widest rendered value so long values stay
    /// readable, but leaves at least a third of the remaining width to
    /// descriptions — unless no row has any info text, in which case values
    /// get all of it. The description column fills whatever is left.
    pub(super) fn param_columns(&self, inner_w: usize) -> (usize, usize, usize) {
        // Min 9 so the "Parameter" column header is never clipped.
        let key_col = self
            .params
            .iter()
            .map(|e| e.key.chars().count())
            .max()
            .unwrap_or(9)
            .clamp(9, (inner_w / 4).max(9));
        let avail = inner_w.saturating_sub(key_col + COLUMN_GAPS);

        let widest_value = self
            .params
            .iter()
            .map(value_display_width)
            .max()
            .unwrap_or(0);
        let has_info = self.params.iter().any(|e| !row_info(e).0.is_empty());
        let value_cap = if has_info {
            avail.saturating_mul(2) / 3
        } else {
            avail
        };
        let value_col = widest_value.clamp(10, value_cap.max(10));
        let desc_col = avail.saturating_sub(value_col);
        (key_col, value_col, desc_col)
    }
}

/// Display width (in columns) of a param's rendered value, including the
/// decorations `value_cell` adds around it (bool symbol, enum position).
fn value_display_width(entry: &ParamEntry) -> usize {
    match &entry.kind {
        ParamKind::Bool => "\u{2717} false".chars().count(),
        ParamKind::Enum(opts) => {
            let n = opts.len();
            entry.value.chars().count() + format!("  ({n}/{n})").chars().count()
        }
        _ => entry.value.chars().count(),
    }
}

/// Build the "Value" cell for a param row. When `editing`, shows the live
/// cursor with horizontal scroll; otherwise a typed widget (bool symbol,
/// wrapped enum value + position, or plain/truncated text).
pub(super) fn value_cell(
    entry: &ParamEntry,
    editing: bool,
    cursor_pos: usize,
    value_width: usize,
) -> Text<'static> {
    let t = theme();
    // Invalid JSON turns the value red (live while typing) so the problem is
    // visible on the value itself, not just in the description column.
    let value_style = if entry.json_valid {
        Style::default().fg(t.text_primary)
    } else {
        Style::default().fg(t.state_failed)
    };

    if editing {
        let (before, cursor_char, after) = value_window(&entry.value, cursor_pos, value_width);
        return Line::from(vec![
            Span::styled(before, value_style),
            // `REVERSED` (not an explicit bg) so the block survives the row
            // highlight, which the Table patches over the cell afterwards.
            Span::styled(
                cursor_char.to_string(),
                value_style.add_modifier(Modifier::REVERSED),
            ),
            Span::styled(after, value_style),
        ])
        .into();
    }

    match &entry.kind {
        ParamKind::Bool => {
            // Booleans use green/red so they read at a glance and don't clash
            // with the accent-colored parameter names.
            let (symbol, color) = if entry.value == "true" {
                ("\u{2713} true", t.state_success)
            } else {
                ("\u{2717} false", t.state_failed)
            };
            Line::from(Span::styled(symbol, Style::default().fg(color))).into()
        }
        ParamKind::Enum(opts) => enum_cell(entry, opts, value_width),
        _ => Line::from(Span::styled(
            truncate_cols(&entry.value, value_width),
            value_style,
        ))
        .into(),
    }
}

/// Enum values wrap (instead of truncating) so the selected option is fully
/// readable; the dimmed `(i/n)` position tag goes on the last line, or on its
/// own line when it doesn't fit.
fn enum_cell(entry: &ParamEntry, opts: &[String], value_width: usize) -> Text<'static> {
    let t = theme();
    let lines = wrap_lines(&entry.value, value_width);

    let mut text: Vec<Line> = lines
        .iter()
        .map(|l| Line::from(Span::styled(l.clone(), Style::default().fg(t.text_primary))))
        .collect();
    if let Some(idx) = opts.iter().position(|o| *o == entry.value) {
        let tag = format!("({}/{})", idx + 1, opts.len());
        let dim = Style::default().fg(t.purple_dim);
        let last_width = lines.last().map_or(0, |l| l.chars().count());
        if last_width + tag.chars().count() + 2 <= value_width {
            if let Some(last) = text.last_mut() {
                last.push_span(Span::styled(format!("  {tag}"), dim));
            }
        } else {
            text.push(Line::from(Span::styled(tag, dim)));
        }
    }
    Text::from(text)
}

/// The "Description" cell: the row's info text wrapped to `width`.
pub(super) fn desc_cell(entry: &ParamEntry, width: usize) -> Text<'static> {
    let (info, style) = row_info(entry);
    Text::from(
        wrap_lines(&info, width)
            .into_iter()
            .map(|l| Line::from(Span::styled(l, style)))
            .collect::<Vec<_>>(),
    )
}

/// Wrap `text` to `width` columns, capped at [`MAX_CELL_LINES`], always
/// returning at least one (possibly empty) line so every row has a height.
fn wrap_lines(text: &str, width: usize) -> Vec<String> {
    let mut lines = wrap_text(text, width);
    lines.truncate(MAX_CELL_LINES);
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// Text + style for a param's "Description" cell: a JSON validity warning
/// takes precedence, else its description, else its option list, else empty.
fn row_info(entry: &ParamEntry) -> (String, Style) {
    let t = theme();
    if !entry.json_valid {
        (
            "\u{26a0} invalid JSON — will be sent as string".to_string(),
            Style::default().fg(t.state_failed),
        )
    } else if let Some(desc) = &entry.description {
        (desc.clone(), Style::default().fg(t.text_primary))
    } else if !entry.options().is_empty() {
        (
            entry.options().join("  |  "),
            Style::default().fg(t.purple_dim),
        )
    } else {
        (String::new(), Style::default().fg(t.text_primary))
    }
}
