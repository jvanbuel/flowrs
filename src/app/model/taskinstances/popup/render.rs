use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::Style,
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget},
};

use crate::{
    app::model::popup::{popup_area, themed_button},
    ui::theme::theme,
};

use super::clear::ClearTaskInstancePopup;
use super::graph::DagGraphPopup;
use super::mark::{MarkState, MarkTaskInstancePopup};

impl Widget for &mut ClearTaskInstancePopup {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let t = theme();
        // Smaller popup: 40% width, auto height
        let area = popup_area(area, 40, 30);

        let popup_block = Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(t.border_style)
            .style(t.surface_style);

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

        let message = if self.task_ids.len() == 1 {
            "Clear this Task Instance?".to_string()
        } else {
            format!("Clear {} Task Instances?", self.task_ids.len())
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

impl Widget for &mut MarkTaskInstancePopup {
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
            .style(t.surface_style);

        let text = Paragraph::new("Mark status as")
            .style(t.default_style)
            .centered();

        let [_, success, _, failed, _, skipped, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(11),
            Constraint::Length(2),
            Constraint::Length(10),
            Constraint::Length(2),
            Constraint::Length(11),
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

        // Skipped button
        let (skipped_style, skipped_border) = if self.status == MarkState::Skipped {
            (t.button_selected, t.border_selected)
        } else {
            (t.button_default, t.border_default)
        };
        let skipped_btn = Paragraph::new("Skipped")
            .style(skipped_style)
            .centered()
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .border_style(skipped_style.fg(skipped_border)),
            );

        Clear.render(area, buffer);
        popup_block.render(area, buffer);
        text.render(header, buffer);
        success_btn.render(success, buffer);
        failed_btn.render(failed, buffer);
        skipped_btn.render(skipped, buffer);
    }
}

impl Widget for &mut DagGraphPopup {
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let t = theme();
        let popup = popup_area(area, 90, 85);

        let block = Block::default()
            .title(" DAG Graph ")
            .title_bottom(" [←↑↓→/hjkl] scroll  [Esc] close ")
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(t.border_style)
            .style(t.surface_style);

        Clear.render(popup, buffer);
        let inner = block.inner(popup);
        block.render(popup, buffer);

        let edge_style = Style::default().fg(t.border_default);

        // Draw edges first (behind nodes)
        for &(src_idx, tgt_idx) in &self.edges {
            let src = &self.nodes[src_idx];
            let tgt = &self.nodes[tgt_idx];

            // Connection points: right-center of source, left-center of target
            let src_x = i32::from(src.x) + i32::from(src.width);
            let src_y = i32::from(src.y) + 1;
            let tgt_x = i32::from(tgt.x);
            let tgt_y = i32::from(tgt.y) + 1;

            if src_y == tgt_y {
                // Same row: straight horizontal line with arrow
                for x in src_x..tgt_x {
                    let sym = if x == tgt_x - 1 { "▶" } else { "─" };
                    self.set_cell(buffer, inner, x, src_y, sym, edge_style);
                }
            } else {
                // Route: horizontal → vertical → horizontal
                let route_x = src_x + (tgt_x - src_x) / 2;

                // Horizontal from source to routing column
                for x in src_x..route_x {
                    self.set_cell(buffer, inner, x, src_y, "─", edge_style);
                }

                if tgt_y > src_y {
                    // Going down
                    self.set_cell(buffer, inner, route_x, src_y, "╮", edge_style);
                    for y in (src_y + 1)..tgt_y {
                        self.set_cell(buffer, inner, route_x, y, "│", edge_style);
                    }
                    self.set_cell(buffer, inner, route_x, tgt_y, "╰", edge_style);
                } else {
                    // Going up
                    self.set_cell(buffer, inner, route_x, src_y, "╯", edge_style);
                    for y in (tgt_y + 1)..src_y {
                        self.set_cell(buffer, inner, route_x, y, "│", edge_style);
                    }
                    self.set_cell(buffer, inner, route_x, tgt_y, "╭", edge_style);
                }

                // Horizontal from routing column to target with arrow
                for x in (route_x + 1)..tgt_x {
                    let sym = if x == tgt_x - 1 { "▶" } else { "─" };
                    self.set_cell(buffer, inner, x, tgt_y, sym, edge_style);
                }
            }
        }

        // Draw nodes (on top of edges)
        let text_style = t.default_style;
        for node in &self.nodes {
            let border_style = Style::default().fg(node.border_color);
            let nx = i32::from(node.x);
            let ny = i32::from(node.y);
            let nw = i32::from(node.width);

            // Top border: ╭───╮
            self.set_cell(buffer, inner, nx, ny, "╭", border_style);
            for dx in 1..nw - 1 {
                self.set_cell(buffer, inner, nx + dx, ny, "─", border_style);
            }
            self.set_cell(buffer, inner, nx + nw - 1, ny, "╮", border_style);

            // Content row: │ name │
            self.set_cell(buffer, inner, nx, ny + 1, "│", border_style);
            // Clear interior
            for dx in 1..nw - 1 {
                self.set_cell(buffer, inner, nx + dx, ny + 1, " ", text_style);
            }
            // Write task name (left-aligned with padding)
            let name_start = nx + 1 + i32::from(super::graph::NODE_PADDING);
            for (i, ch) in node.task_id.chars().enumerate() {
                self.set_cell(
                    buffer,
                    inner,
                    name_start + i as i32,
                    ny + 1,
                    &ch.to_string(),
                    text_style,
                );
            }
            self.set_cell(buffer, inner, nx + nw - 1, ny + 1, "│", border_style);

            // Bottom border: ╰───╯
            self.set_cell(buffer, inner, nx, ny + 2, "╰", border_style);
            for dx in 1..nw - 1 {
                self.set_cell(buffer, inner, nx + dx, ny + 2, "─", border_style);
            }
            self.set_cell(buffer, inner, nx + nw - 1, ny + 2, "╯", border_style);
        }
    }
}
