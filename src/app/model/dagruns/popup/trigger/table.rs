//! Cell contents and column sizing for the trigger popup's param table.

use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use crate::ui::theme::theme;

use super::params::{ParamEntry, ParamKind};
use super::text::{truncate_cols, value_window};
use super::TriggerDagRunPopUp;

/// Total inter-column padding the table reserves (1 col between each of 3 columns).
const COLUMN_GAPS: usize = 2;

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
        let key_col = self
            .params
            .iter()
            .map(|e| e.key.chars().count())
            .max()
            .unwrap_or(8)
            .clamp(8, (inner_w / 4).max(8));
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
/// enum value + position, or plain/truncated text).
pub(super) fn value_cell(
    entry: &ParamEntry,
    editing: bool,
    cursor_pos: usize,
    value_width: usize,
) -> Line<'static> {
    let t = theme();

    if editing {
        let (before, cursor_char, after) = value_window(&entry.value, cursor_pos, value_width);
        return Line::from(vec![
            Span::styled(before, Style::default().fg(t.text_primary)),
            // `REVERSED` (not an explicit bg) so the block survives the row
            // highlight, which the Table patches over the cell afterwards.
            Span::styled(
                cursor_char.to_string(),
                Style::default()
                    .fg(t.text_primary)
                    .add_modifier(Modifier::REVERSED),
            ),
            Span::styled(after, Style::default().fg(t.text_primary)),
        ]);
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
            Line::from(Span::styled(symbol, Style::default().fg(color)))
        }
        ParamKind::Enum(opts) => {
            let position = opts
                .iter()
                .position(|o| *o == entry.value)
                .map(|idx| format!("  ({}/{})", idx + 1, opts.len()))
                .unwrap_or_default();
            let value = truncate_cols(
                &entry.value,
                value_width.saturating_sub(position.chars().count()),
            );
            Line::from(vec![
                Span::styled(value, Style::default().fg(t.text_primary)),
                Span::styled(position, Style::default().fg(t.purple_dim)),
            ])
        }
        _ => Line::from(Span::styled(
            truncate_cols(&entry.value, value_width),
            Style::default().fg(t.text_primary),
        )),
    }
}

/// Text + style for a param's "Description" cell: its description, else a JSON
/// validity warning, else its option list, else empty.
pub(super) fn row_info(entry: &ParamEntry) -> (String, Style) {
    let t = theme();
    if let Some(desc) = &entry.description {
        (desc.clone(), Style::default().fg(t.text_primary))
    } else if !entry.json_valid {
        (
            "\u{26a0} invalid JSON — will be sent as string".to_string(),
            Style::default().fg(t.state_failed),
        )
    } else if !entry.options().is_empty() {
        (
            entry.options().join("  |  "),
            Style::default().fg(t.purple_dim),
        )
    } else {
        (String::new(), Style::default().fg(t.text_primary))
    }
}
