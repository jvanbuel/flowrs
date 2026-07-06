//! Rendering of the trigger popup: a simple confirmation when the DAG has no
//! params, or an editable param table when it does.

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
    app::model::popup::{popup_area, render_yes_no},
    ui::theme::theme,
};

use super::table::{row_info, value_cell};
use super::text::wrap_text;
use super::{FocusZone, TriggerDagRunPopUp};

/// Popup width as a percent of the screen; the height grows with content.
const PARAMS_POPUP_WIDTH_PCT: u16 = 75;
/// Cap on popup height as a percent of the screen (content scrolls beyond it).
const PARAMS_POPUP_MAX_H_PCT: u16 = 85;
/// Max wrapped lines shown per param description (taller rows scroll the table).
const MAX_DESC_LINES: usize = 3;

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

        let active = self.active_param;
        let editing = self.editing && self.focus == FocusZone::Params;

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

        // Pre-build the value cells: enum values wrap, so they contribute to
        // row height too.
        let value_cells: Vec<Text> = self
            .params
            .iter()
            .enumerate()
            .map(|(i, entry)| value_cell(entry, editing && i == active, self.cursor_pos, value_col))
            .collect();

        let total_rows_h: u16 = infos
            .iter()
            .zip(value_cells.iter())
            .map(|((lines, _), value)| u16::try_from(lines.len().max(value.height())).unwrap_or(1))
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

        let rows: Vec<Row> = self
            .params
            .iter()
            .zip(infos.iter().zip(value_cells))
            .map(|(entry, ((lines, style), value))| {
                let height = u16::try_from(lines.len().max(value.height())).unwrap_or(1);
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
                    Cell::from(value),
                    Cell::from(desc),
                ])
                .height(height)
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
}

/// Center a `width` × `height` rect within `area` (clamped to fit).
fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    let x = area.x + (area.width - width) / 2;
    let y = area.y + (area.height - height) / 2;
    Rect::new(x, y, width, height)
}

#[cfg(test)]
mod tests {
    use crate::airflow::model::common::DagId;
    use crate::app::model::dagruns::popup::trigger::TriggerDagRunPopUp;
    use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

    fn buffer_text(buf: &Buffer) -> String {
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
