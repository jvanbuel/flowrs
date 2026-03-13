use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Row, StatefulWidget, Table, Widget};
use time::format_description;

use crate::airflow::model::common::{calculate_duration, format_duration};
use crate::ui::common::create_headers;
use crate::ui::constants::AirflowStateColor;
use crate::ui::theme::{BORDER_STYLE, SELECTED_ROW_STYLE, TABLE_HEADER_STYLE};
use crate::ui::TIME_FORMAT;

use super::popup::DagRunPopUp;
use super::DagRunModel;

impl Widget for &mut DagRunModel {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let content_area = self.table.render_with_filter(area, buf);

        let headers = [
            "State",
            "DAG Run ID",
            "Logical Date",
            "Type",
            "Duration",
            "Time",
        ];
        let header_row = create_headers(headers);
        let header = Row::new(header_row).style(TABLE_HEADER_STYLE);

        // Calculate max duration for normalization
        let max_duration = self
            .table
            .filtered
            .items
            .iter()
            .filter_map(calculate_duration)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(1.0);

        // Calculate the width available for the Duration column
        // Total width - borders(2) - state(6) - dag_run_id(variable) - logical_date(20) - type(11) - time(10)
        let table_inner_width = content_area.width.saturating_sub(2); // Subtract borders
        let fixed_columns_width = 6 + 20 + 11 + 10 + 10; // State + Logical Date + Type + Time + spacing
        let dag_run_id_width = 30; // Fixed width for dag_run_id
        let gauge_width = table_inner_width
            .saturating_sub(fixed_columns_width + dag_run_id_width)
            .max(10) as usize;

        let rows = self
            .table
            .filtered
            .items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                let state_color: Color = AirflowStateColor::from(&item.state).into();

                let (duration_cell, time_cell) = if let Some(duration) = calculate_duration(item) {
                    (
                        DagRunModel::create_duration_gauge(
                            duration,
                            max_duration,
                            state_color,
                            gauge_width,
                        ),
                        Line::from(format_duration(duration)),
                    )
                } else {
                    (Line::from("-"), Line::from("-"))
                };

                Row::new(vec![
                    Line::from(Span::styled("■", Style::default().fg(state_color))),
                    Line::from(Span::styled(
                        &*item.dag_run_id,
                        Style::default().add_modifier(Modifier::BOLD),
                    )),
                    Line::from(if let Some(date) = item.logical_date {
                        date.format(
                            &format_description::parse(TIME_FORMAT)
                                .expect("TIME_FORMAT constant should be a valid time format"),
                        )
                        .expect("Date formatting with TIME_FORMAT should succeed")
                    } else {
                        "None".to_string()
                    }),
                    Line::from(item.run_type.to_string()),
                    duration_cell,
                    time_cell,
                ])
                .style(self.table.row_style(idx))
            });
        let t = Table::new(
            rows,
            &[
                Constraint::Length(6),  // State
                Constraint::Length(30), // DAG Run ID
                Constraint::Length(20), // Logical Date
                Constraint::Length(11), // Type
                Constraint::Fill(1),    // Duration gauge (expands)
                Constraint::Length(10), // Time (formatted duration)
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
        StatefulWidget::render(t, content_area, buf, &mut self.table.filtered.state);

        if let Some(view) = &mut self.dag_code {
            view.render(area, buf);
        }

        // Render any active popup (error, commands, or custom)
        (&self.popup).render(area, buf);

        // Render custom popups that need special handling
        match self.popup.custom_mut() {
            Some(DagRunPopUp::Clear(popup)) => popup.render(area, buf),
            Some(DagRunPopUp::Mark(popup)) => popup.render(area, buf),
            Some(DagRunPopUp::Trigger(popup)) => popup.render(area, buf),
            None => {}
        }
    }
}
