use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, Row, StatefulWidget, Table, Widget};

use crate::airflow::model::common::{calculate_duration, format_duration};
use crate::ui::common::{create_headers, state_to_colored_square};
use crate::ui::constants::AirflowStateColor;
use crate::ui::gantt::create_gantt_bar;
use crate::ui::theme::{BORDER_STYLE, SELECTED_ROW_STYLE, TABLE_HEADER_STYLE};

use super::popup::TaskInstancePopUp;
use super::TaskInstanceModel;

impl Widget for &mut TaskInstanceModel {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let content_area = self.table.render_with_filter(area, buffer);

        let headers = ["Task ID", "Duration", "State", "Tries", "Gantt"];
        let header_row = create_headers(headers);
        let header = Row::new(header_row).style(TABLE_HEADER_STYLE);

        // Calculate the width available for the Gantt column (capped at half the panel)
        let table_inner_width = content_area.width.saturating_sub(2); // Subtract borders
        let gantt_width = (table_inner_width / 2).max(10);

        let rows = self
            .table
            .filtered
            .items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                Row::new(vec![
                    Line::from(item.task_id.as_ref()),
                    Line::from(
                        calculate_duration(item).map_or_else(|| "-".to_string(), format_duration),
                    ),
                    Line::from(state_to_colored_square(
                        item.state
                            .as_ref()
                            .map_or(AirflowStateColor::None, AirflowStateColor::from),
                    )),
                    Line::from(item.try_number.to_string()),
                    create_gantt_bar(&self.gantt_data, &item.task_id, gantt_width.into()),
                ])
                .style(self.table.row_style(idx))
            });
        let t = Table::new(
            rows,
            &[
                Constraint::Fill(1),             // Task ID (expands)
                Constraint::Length(10),          // Duration
                Constraint::Length(5),           // State
                Constraint::Length(5),           // Tries
                Constraint::Length(gantt_width), // Gantt chart (capped at half)
            ],
        )
        .header(header)
        .block({
            let block = Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                .border_style(BORDER_STYLE)
                .title(" Press <?> to see available commands ");
            if let Some(title) = self.table.status_title() {
                block.title_bottom(title)
            } else {
                block
            }
        })
        .row_highlight_style(SELECTED_ROW_STYLE);

        StatefulWidget::render(t, content_area, buffer, &mut self.table.filtered.state);

        // Render any active popup (error, commands, or custom)
        (&self.popup).render(area, buffer);

        // Render custom popups that need special handling
        match self.popup.custom_mut() {
            Some(TaskInstancePopUp::Clear(popup)) => popup.render(area, buffer),
            Some(TaskInstancePopUp::Mark(popup)) => popup.render(area, buffer),
            None => {}
        }
    }
}
