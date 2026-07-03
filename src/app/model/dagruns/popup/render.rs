use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, Paragraph, Row, StatefulWidget, Table, Widget,
    },
};

use crate::{
    app::model::popup::{popup_area, themed_button},
    ui::theme::theme,
};

use super::clear::ClearDagRunPopup;
use super::mark::{MarkDagRunPopup, MarkState};
use super::trigger::{FocusZone, ParamEntry, ParamKind, TriggerDagRunPopUp};

/// Popup width as a percent of the screen; the height grows with content.
const PARAMS_POPUP_WIDTH_PCT: u16 = 75;
/// Cap on popup height as a percent of the screen (content scrolls beyond it).
const PARAMS_POPUP_MAX_H_PCT: u16 = 85;
/// Max wrapped lines shown per param description (taller rows scroll the table).
const MAX_DESC_LINES: usize = 3;
/// Total inter-column padding the table reserves (1 col between each of 3 columns).
const COLUMN_GAPS: usize = 2;

impl Widget for &mut TriggerDagRunPopUp {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        if self.has_params() {
            self.render_with_params(area, buffer);
        } else {
            self.render_simple(area, buffer);
        }
    }
}

impl TriggerDagRunPopUp {
    fn render_simple(&mut self, area: Rect, buffer: &mut Buffer) {
        let t = theme();
        // Smaller popup: 40% width, auto height
        let area = popup_area(area, 40, 30);

        let popup_block = Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(t.border_style)
            .style(t.default_style);

        // Use inner area for content layout to avoid overlapping the border
        let inner = popup_block.inner(area);

        let [_, header, options, _] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .flex(Flex::Center)
        .areas(inner);

        let text = Paragraph::new("Trigger a new DAG Run?")
            .style(t.default_style)
            .centered();

        Clear.render(area, buffer);
        popup_block.render(area, buffer);
        text.render(header, buffer);
        render_yes_no(options, buffer, self.selected_button.is_yes(), true);
    }

    fn render_with_params(&mut self, area: Rect, buffer: &mut Buffer) {
        let t = theme();

        // Fixed width (percent of screen); the height grows with the content.
        let popup_w = area.width.saturating_mul(PARAMS_POPUP_WIDTH_PCT) / 100;
        let inner_w = popup_w.saturating_sub(2) as usize;

        let (key_col, value_col, desc_col) = self.param_columns(inner_w);

        // Pre-wrap each description so a row is as tall as its (capped) wrap.
        let infos: Vec<(Vec<String>, Style)> = self
            .params
            .iter()
            .map(|entry| {
                let (text, style) = row_info(entry);
                let mut lines = wrap_text(&text, desc_col);
                lines.truncate(MAX_DESC_LINES);
                if lines.is_empty() {
                    lines.push(String::new());
                }
                (lines, style)
            })
            .collect();

        let total_rows_h: u16 = infos
            .iter()
            .map(|(lines, _)| u16::try_from(lines.len()).unwrap_or(1))
            .sum();
        // chrome = header(1) + spacer(1) + buttons(3) + legend(1) + table header(1) + borders(2)
        let desired_h = total_rows_h.saturating_add(9);
        let max_h = area.height.saturating_mul(PARAMS_POPUP_MAX_H_PCT) / 100;
        let popup_h = desired_h.clamp(10, max_h.max(10));
        let area = centered_rect(area, popup_w, popup_h);

        let popup_block = Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(t.border_style)
            .style(t.default_style)
            .title(Span::styled(
                " Trigger DAG Run ",
                Style::default()
                    .fg(t.text_primary)
                    .add_modifier(Modifier::BOLD),
            ));
        let inner = popup_block.inner(area);

        // header · param table · spacer · buttons · key legend. The table gets
        // `Fill` (lowest priority) so that on very short screens it shrinks
        // first and the buttons keep their 3 rows — with `Min(3)` the solver
        // squeezed the buttons instead, rendering them as empty boxes.
        let [header_area, table_area, _, buttons_area, legend_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .areas(inner);

        Clear.render(area, buffer);
        popup_block.render(area, buffer);

        Paragraph::new("Edit parameters and confirm:")
            .style(t.default_style)
            .centered()
            .render(header_area, buffer);

        let active = self.active_param;
        let editing = self.editing && self.focus == FocusZone::Params;
        let rows: Vec<Row> = self
            .params
            .iter()
            .zip(infos.iter())
            .enumerate()
            .map(|(i, (entry, (lines, style)))| {
                let desc = Text::from(
                    lines
                        .iter()
                        .map(|l| Line::from(Span::styled(l.clone(), *style)))
                        .collect::<Vec<_>>(),
                );
                Row::new(vec![
                    // Parameter names in the accent color to set them apart
                    // from the (neutral) values.
                    Cell::from(Span::styled(
                        entry.key.clone(),
                        Style::default().fg(t.accent),
                    )),
                    Cell::from(value_cell(
                        entry,
                        editing && i == active,
                        self.cursor_pos,
                        value_col,
                    )),
                    Cell::from(desc),
                ])
                .height(u16::try_from(lines.len()).unwrap_or(1))
            })
            .collect();

        let header_row =
            Row::new(["Parameter", "Value", "Description"]).style(t.table_header_style);
        let table = Table::new(
            rows,
            [
                Constraint::Length(u16::try_from(key_col).unwrap_or(u16::MAX)),
                Constraint::Length(u16::try_from(value_col).unwrap_or(u16::MAX)),
                Constraint::Fill(1),
            ],
        )
        .header(header_row)
        .column_spacing(1)
        .row_highlight_style(t.selected_row_style);

        self.table_state.select(Some(active));
        StatefulWidget::render(table, table_area, buffer, &mut self.table_state);

        // Buttons (highlight only when the buttons zone is focused).
        render_yes_no(
            buttons_area,
            buffer,
            self.selected_button.is_yes(),
            self.focus == FocusZone::Buttons,
        );

        // Context-aware key legend (the controls aren't otherwise discoverable).
        let legend = match (self.editing, self.focus) {
            (true, _) => "type to edit  ·  Enter/Esc/Tab done",
            (false, FocusZone::Params) => {
                "↑↓ move  ·  Enter edit  ·  Space toggle/cycle  ·  Tab → buttons  ·  Esc cancel"
            }
            (false, FocusZone::Buttons) => "←→ Yes/No  ·  Enter confirm  ·  Tab/Esc → params",
        };
        Paragraph::new(legend)
            .style(Style::default().fg(t.purple_dim))
            .centered()
            .render(legend_area, buffer);
    }

    /// Compute the (key, value, description) column widths for a popup of
    /// `inner_w` columns.
    ///
    /// The key column is content-sized, capped at a quarter of the popup. The
    /// value column grows with its widest rendered value so long values stay
    /// readable, but leaves at least a third of the remaining width to
    /// descriptions — unless no row has any info text, in which case values
    /// get all of it. The description column fills whatever is left.
    fn param_columns(&self, inner_w: usize) -> (usize, usize, usize) {
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
fn value_cell(
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
fn row_info(entry: &ParamEntry) -> (String, Style) {
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

/// Greedy word-wrap of `text` to `width` columns. Long words are hard-split.
fn wrap_text(text: &str, width: usize) -> Vec<String> {
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

/// Center a `width` × `height` rect within `area` (clamped to fit).
fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    let x = area.x + (area.width - width) / 2;
    let y = area.y + (area.height - height) / 2;
    Rect::new(x, y, width, height)
}

/// Render the standard centered Yes / No button pair into `area`.
/// `active` dims the highlight when the buttons aren't the focused zone.
fn render_yes_no(area: Rect, buffer: &mut Buffer, yes_selected: bool, active: bool) {
    if area.height == 0 {
        return;
    }
    let [_, yes, _, no, _] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(8),
        Constraint::Length(2),
        Constraint::Length(8),
        Constraint::Fill(1),
    ])
    .areas(area);
    if area.height >= 3 {
        themed_button("Yes", active && yes_selected).render(yes, buffer);
        themed_button("No", active && !yes_selected).render(no, buffer);
    } else {
        // Too short for bordered buttons — the border would swallow the label
        // and leave two blank boxes. Fall back to flat highlighted labels.
        let t = theme();
        let style = |selected: bool| {
            if selected {
                t.button_selected
            } else {
                t.button_default
            }
        };
        Paragraph::new("Yes")
            .style(style(active && yes_selected))
            .centered()
            .render(Rect { height: 1, ..yes }, buffer);
        Paragraph::new("No")
            .style(style(active && !yes_selected))
            .centered()
            .render(Rect { height: 1, ..no }, buffer);
    }
}

/// Slide a `width`-column window over `value` so the char at `cursor_pos`
/// (a byte index) stays visible. Returns the text before the cursor, the
/// cursor char (a space if the cursor is at the end), and the text after.
fn value_window(value: &str, cursor_pos: usize, width: usize) -> (String, char, String) {
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
fn truncate_cols(s: &str, max_cols: usize) -> String {
    if s.chars().count() <= max_cols {
        return s.to_string();
    }
    if max_cols == 0 {
        return String::new();
    }
    let kept: String = s.chars().take(max_cols - 1).collect();
    format!("{kept}…")
}

impl Widget for &mut ClearDagRunPopup {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let t = theme();
        // Smaller popup: 40% width, auto height
        let area = popup_area(area, 40, 30);

        let popup_block = Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(t.border_style)
            .style(t.default_style);

        // Use inner area for content layout to avoid overlapping the border
        let inner = popup_block.inner(area);

        let [_, header, options, _] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .flex(Flex::Center)
        .areas(inner);

        let message = if self.dag_run_ids.len() == 1 {
            "Clear this DAG Run?".to_string()
        } else {
            format!("Clear {} DAG Runs?", self.dag_run_ids.len())
        };
        let text = Paragraph::new(message).style(t.default_style).centered();

        Clear.render(area, buffer);
        popup_block.render(area, buffer);
        text.render(header, buffer);
        render_yes_no(options, buffer, self.selected_button.is_yes(), true);
    }
}

impl Widget for &mut MarkDagRunPopup {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let t = theme();
        // Smaller popup: 35% width, auto height
        let area = popup_area(area, 35, 30);

        let [_, header, options, _] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .flex(Flex::Center)
        .areas(area);

        let popup_block = Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(t.border_style)
            .style(t.default_style);

        let text = Paragraph::new("Mark status as")
            .style(t.default_style)
            .centered();

        let [_, success, _, failed, _, queued, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(11),
            Constraint::Length(2),
            Constraint::Length(10),
            Constraint::Length(2),
            Constraint::Length(10),
            Constraint::Fill(1),
        ])
        .areas(options);

        // Success button
        let (success_style, success_border) = if self.status == MarkState::Success {
            (t.button_selected, t.border_selected)
        } else {
            (t.button_default, t.border_default)
        };
        let success_btn = Paragraph::new("Success")
            .style(success_style)
            .centered()
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .border_style(success_style.fg(success_border)),
            );

        // Failed button
        let (failed_style, failed_border) = if self.status == MarkState::Failed {
            (t.button_selected, t.border_selected)
        } else {
            (t.button_default, t.border_default)
        };
        let failed_btn = Paragraph::new("Failed")
            .style(failed_style)
            .centered()
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .border_style(failed_style.fg(failed_border)),
            );

        // Queued button
        let (queued_style, queued_border) = if self.status == MarkState::Queued {
            (t.button_selected, t.border_selected)
        } else {
            (t.button_default, t.border_default)
        };
        let queued_btn = Paragraph::new("Queued")
            .style(queued_style)
            .centered()
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .border_style(queued_style.fg(queued_border)),
            );

        Clear.render(area, buffer);
        popup_block.render(area, buffer);
        text.render(header, buffer);
        success_btn.render(success, buffer);
        failed_btn.render(failed, buffer);
        queued_btn.render(queued, buffer);
    }
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

    fn buffer_text(buf: &ratatui::buffer::Buffer) -> String {
        let area = buf.area;
        let mut out = String::new();
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                out.push_str(buf[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    #[test]
    fn yes_no_buttons_survive_cramped_heights() {
        // Regression: on short screens the layout solver used to squeeze the
        // buttons row below 3 lines, so the bordered Yes/No boxes rendered as
        // empty borders with no label.
        use crate::airflow::model::common::DagId;
        use crate::app::model::dagruns::popup::trigger::TriggerDagRunPopUp;
        use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

        crate::ui::theme::init_theme(flowrs_config::Theme::Dark);

        let schema = serde_json::json!({
            "a": {"value": "1", "schema": {"type": "string"}},
            "b": {"value": "2", "schema": {"type": "string"}},
            "c": {"value": "3", "schema": {"type": "string"}},
        });
        let mut popup = TriggerDagRunPopUp::new(DagId::from("d"), Some(&schema));

        for height in [6u16, 8, 10, 12, 24] {
            let area = Rect::new(0, 0, 60, height);
            let mut buf = Buffer::empty(area);
            (&mut popup).render(area, &mut buf);
            let text = buffer_text(&buf);
            assert!(text.contains("Yes"), "no Yes button at height {height}");
            assert!(text.contains("No"), "no No button at height {height}");
        }
    }

    #[test]
    fn renders_param_table_without_panicking() {
        use crate::airflow::model::common::DagId;
        use crate::app::model::dagruns::popup::trigger::TriggerDagRunPopUp;
        use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

        crate::ui::theme::init_theme(flowrs_config::Theme::Dark);

        let schema = serde_json::json!({
            "flag": {"value": true, "description": "a flag", "schema": {"type": "boolean"}},
            "name": {"value": "abc", "schema": {"type": "string"}},
            "mode": {"value": "a", "schema": {"enum": ["a", "b", "c"]}},
            "long": {
                "value": "x",
                "description": "A deliberately long description that must wrap across \
                    several lines so the multi-line description column and the row \
                    height calculation get exercised by the smoke test.",
                "schema": {"type": "string"}
            },
        });
        let mut popup = TriggerDagRunPopUp::new(DagId::from("d"), Some(&schema));

        // Render at a normal size and a cramped one (where the width math is
        // most likely to underflow), plus in editing mode.
        for area in [Rect::new(0, 0, 80, 24), Rect::new(0, 0, 12, 6)] {
            let mut buf = Buffer::empty(area);
            (&mut popup).render(area, &mut buf);
        }
        popup.editing = true;
        popup.cursor_pos = 0;
        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        (&mut popup).render(area, &mut buf);
    }
}
