use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget},
};

use crate::{
    app::model::popup::{popup_area, themed_button},
    ui::theme::theme,
};

use super::clear::ClearDagRunPopup;
use super::mark::{MarkDagRunPopup, MarkState};
use super::trigger::{FocusZone, ParamEntry, ParamKind, TriggerDagRunPopUp};

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

        let [_, yes, _, no, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(8),
            Constraint::Length(2),
            Constraint::Length(8),
            Constraint::Fill(1),
        ])
        .areas(options);

        let yes_btn = themed_button("Yes", self.selected_button.is_yes());
        let no_btn = themed_button("No", !self.selected_button.is_yes());

        Clear.render(area, buffer);
        popup_block.render(area, buffer);
        text.render(header, buffer);
        yes_btn.render(yes, buffer);
        no_btn.render(no, buffer);
    }

    fn render_with_params(&mut self, area: Rect, buffer: &mut Buffer) {
        let t = theme();
        let area = popup_area(area, 60, 70);

        let popup_block = Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(t.border_style)
            .style(t.surface_style)
            .title(" Trigger DAG Run ");

        let inner = popup_block.inner(area);

        // Each param gets 2 rows: value line + description/options ghost line
        let [header_area, params_area, _, buttons_area, _] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .areas(inner);

        let header = Paragraph::new("Edit parameters and confirm:")
            .style(t.default_style)
            .centered();

        let rows_per_param: usize = 2;
        let visible_params = params_area.height as usize / rows_per_param;

        // Scroll handling
        if self.active_param >= self.scroll_offset + visible_params {
            self.scroll_offset = self.active_param + 1 - visible_params;
        }
        if self.active_param < self.scroll_offset {
            self.scroll_offset = self.active_param;
        }

        Clear.render(area, buffer);
        popup_block.render(area, buffer);
        header.render(header_area, buffer);

        let ghost_style = Style::default().fg(t.purple_dim);

        let max_key_len = self.params.iter().map(|e| e.key.len()).max().unwrap_or(0);

        for (row_idx, (i, entry)) in self
            .params
            .iter()
            .enumerate()
            .skip(self.scroll_offset)
            .take(visible_params)
            .enumerate()
        {
            let Some(row_offset) = u16::try_from(row_idx * rows_per_param).ok() else {
                break;
            };
            let row_y = params_area.y + row_offset;
            if row_y + 1 >= params_area.y + params_area.height {
                break;
            }
            let row_area = Rect::new(
                params_area.x + 1,
                row_y,
                params_area.width.saturating_sub(2),
                1,
            );
            let ghost_area = Rect::new(
                params_area.x + 1,
                row_y + 1,
                params_area.width.saturating_sub(2),
                1,
            );

            let is_active = i == self.active_param && self.focus == FocusZone::Params;
            let key_style = if is_active {
                Style::default().fg(t.accent)
            } else {
                Style::default().fg(t.text_primary)
            };

            let truncated_key = if entry.key.len() > max_key_len {
                let char_boundary = entry
                    .key
                    .char_indices()
                    .take_while(|(i, _)| *i < max_key_len.saturating_sub(1))
                    .last()
                    .map_or(0, |(i, c)| i + c.len_utf8());
                format!("{}…", &entry.key[..char_boundary])
            } else {
                format!("{:max_key_len$}", entry.key)
            };

            let value_line = render_value_line(
                self,
                entry,
                &truncated_key,
                key_style,
                ghost_style,
                is_active,
            );
            value_line.render(row_area, buffer);

            // Ghost line: description, or options list, or kind hint
            let ghost_line = render_ghost_line(entry, max_key_len, ghost_style, is_active);
            if let Some(line) = ghost_line {
                line.render(ghost_area, buffer);
            }
        }

        // Buttons
        let [_, yes, _, no, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(8),
            Constraint::Length(2),
            Constraint::Length(8),
            Constraint::Fill(1),
        ])
        .areas(buttons_area);

        let btn_focus = self.focus == FocusZone::Buttons;
        let yes_btn = themed_button("Yes", btn_focus && self.selected_button.is_yes());
        let no_btn = themed_button("No", btn_focus && !self.selected_button.is_yes());

        yes_btn.render(yes, buffer);
        no_btn.render(no, buffer);
    }
}

fn render_value_line(
    popup: &TriggerDagRunPopUp,
    entry: &ParamEntry,
    truncated_key: &str,
    key_style: Style,
    ghost_style: Style,
    is_active: bool,
) -> Line<'static> {
    let t = theme();

    if is_active && popup.editing {
        return render_editing_line(popup, entry, truncated_key, key_style);
    }

    let bracket_style = if is_active {
        Style::default().fg(t.accent)
    } else {
        Style::default().fg(t.purple_dim)
    };

    let mut spans = vec![Span::styled(format!("{truncated_key}: "), key_style)];

    match &entry.kind {
        ParamKind::Bool => {
            let (symbol, color) = if entry.value == "true" {
                ("\u{2713} true", t.accent)
            } else {
                ("\u{2717} false", t.purple_dim)
            };
            spans.push(Span::styled(symbol.to_string(), Style::default().fg(color)));
            if is_active {
                spans.push(Span::styled(" <Space> toggle", ghost_style));
            }
        }
        ParamKind::Enum(opts) => {
            let current_idx = opts.iter().position(|o| *o == entry.value);
            spans.push(Span::styled("[", bracket_style));
            spans.push(Span::styled(
                entry.value.clone(),
                Style::default().fg(t.text_primary),
            ));
            spans.push(Span::styled("]", bracket_style));
            if let Some(idx) = current_idx {
                spans.push(Span::styled(
                    format!(" ({}/{})", idx + 1, opts.len()),
                    ghost_style,
                ));
            }
            if is_active {
                spans.push(Span::styled(" <Space> cycle", ghost_style));
            }
        }
        _ => {
            spans.push(Span::styled("[", bracket_style));
            spans.push(Span::styled(
                entry.value.clone(),
                Style::default().fg(t.text_primary),
            ));
            spans.push(Span::styled("]", bracket_style));
            if is_active && entry.has_options() {
                spans.push(Span::styled(" <Space> cycle", ghost_style));
            }
        }
    }

    Line::from(spans)
}

fn render_editing_line(
    popup: &TriggerDagRunPopUp,
    entry: &ParamEntry,
    truncated_key: &str,
    key_style: Style,
) -> Line<'static> {
    let t = theme();
    let (before, after) = entry
        .value
        .split_at(popup.cursor_pos.min(entry.value.len()));
    let cursor_char = after.chars().next().unwrap_or(' ');
    let rest = if after.is_empty() {
        String::new()
    } else {
        after[cursor_char.len_utf8()..].to_string()
    };
    Line::from(vec![
        Span::styled(format!("{truncated_key}: "), key_style),
        Span::styled(before.to_string(), Style::default().fg(t.text_primary)),
        Span::styled(
            cursor_char.to_string(),
            Style::default().fg(t.surface).bg(t.text_primary),
        ),
        Span::styled(rest, Style::default().fg(t.text_primary)),
    ])
}

fn render_ghost_line(
    entry: &ParamEntry,
    key_padding: usize,
    ghost_style: Style,
    is_active: bool,
) -> Option<Line<'static>> {
    let padding = " ".repeat(key_padding + 2); // align with value after "key: "

    // Description always takes priority
    if let Some(desc) = &entry.description {
        return Some(Line::from(Span::styled(
            format!("{padding}{desc}"),
            ghost_style,
        )));
    }

    // Show JSON parse warning when value is invalid
    if !entry.json_valid {
        let t = theme();
        return Some(Line::from(Span::styled(
            format!("{padding}\u{26a0} invalid JSON — will be sent as string"),
            Style::default().fg(t.state_failed),
        )));
    }

    // For active entries with options, show the option list
    if is_active {
        let opts = entry.options();
        if !opts.is_empty() {
            let opts_display: Vec<String> = opts
                .iter()
                .map(|o| {
                    if *o == entry.value {
                        format!("[{o}]")
                    } else {
                        o.clone()
                    }
                })
                .collect();
            return Some(Line::from(Span::styled(
                format!("{padding}{}", opts_display.join(" | ")),
                ghost_style,
            )));
        }
    }

    None
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

        let [_, yes, _, no, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(8),
            Constraint::Length(2),
            Constraint::Length(8),
            Constraint::Fill(1),
        ])
        .areas(options);

        let yes_btn = themed_button("Yes", self.selected_button.is_yes());
        let no_btn = themed_button("No", !self.selected_button.is_yes());

        Clear.render(area, buffer);
        popup_block.render(area, buffer);
        text.render(header, buffer);
        yes_btn.render(yes, buffer);
        no_btn.render(no, buffer);
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
