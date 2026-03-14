use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Row, StatefulWidget, Table, Widget};
use time::OffsetDateTime;

use crate::airflow::model::common::DagRunState;
use crate::ui::common::create_headers;
use crate::ui::constants::AirflowStateColor;
use crate::ui::theme::theme;

use super::popup::DagPopUp;
use super::DagModel;

impl Widget for &mut DagModel {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let content_area = self.table.render_with_filter(area, buf);
        let t = theme();

        let headers = ["Active", "Name", "Owners", "Schedule", "Next Run", "Stats"];
        let header_row = create_headers(headers);
        let header = Row::new(header_row).style(t.table_header_style);
        let rows = self
            .table
            .filtered
            .items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                Row::new(vec![
                    if item.is_paused {
                        Line::from(Span::styled("𖣘", Style::default().fg(t.text_primary)))
                    } else {
                        Line::from(Span::styled("𖣘", Style::default().fg(t.dag_active)))
                    },
                    Line::from(Span::styled(
                        &*item.dag_id,
                        Style::default().add_modifier(Modifier::BOLD),
                    )),
                    Line::from(item.owners.join(", ")),
                    Line::from(item.timetable_description.as_deref().unwrap_or("None"))
                        .style(Style::default().fg(Color::LightYellow)),
                    Line::from(item.next_dagrun_create_after.map_or_else(
                        || "None".to_string(),
                        convert_datetimeoffset_to_human_readable_remaining_time,
                    )),
                    Line::from(self.dag_stats.get(&item.dag_id).map_or_else(
                        || vec![Span::styled("None".to_string(), Style::default())],
                        |stats| {
                            stats
                                .iter()
                                .map(|stat| {
                                    Span::styled(
                                        format!("{:>7}", stat.count),
                                        match (&stat.state, stat.count) {
                                            (DagRunState::Running | DagRunState::Failed, 0) => {
                                                Style::default().fg(AirflowStateColor::None.into())
                                            }
                                            _ => Style::default()
                                                .fg(AirflowStateColor::from(&stat.state).into()),
                                        },
                                    )
                                })
                                .collect::<Vec<Span>>()
                        },
                    )),
                ])
                .style(self.table.row_style(idx))
            });
        let t = Table::new(
            rows,
            &[
                Constraint::Length(6),
                Constraint::Fill(2),
                Constraint::Max(20),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(30),
            ],
        )
        .header(header)
        .block({
            let block = Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                .border_style(t.border_style)
                .title(" Press <?> to see available commands ");
            if let Some(title) = self.table.status_title() {
                block.title_bottom(title)
            } else {
                block
            }
        })
        .row_highlight_style(t.selected_row_style);

        StatefulWidget::render(t, content_area, buf, &mut self.table.filtered.state);

        if let Some(view) = &mut self.dag_code {
            view.render(area, buf);
        }

        // Render any active popup (error, commands, or custom)
        (&self.popup).render(area, buf);

        // Render custom popups that need special handling
        if let Some(DagPopUp::Trigger(trigger_popup)) = self.popup.custom_mut() {
            trigger_popup.render(area, buf);
        }
    }
}

fn convert_datetimeoffset_to_human_readable_remaining_time(dt: OffsetDateTime) -> String {
    let now = OffsetDateTime::now_utc();
    let duration = dt.unix_timestamp() - now.unix_timestamp();
    #[allow(clippy::cast_sign_loss)]
    let duration = if duration < 0 { 0 } else { duration as u64 };
    let days = duration / (24 * 3600);
    let hours = (duration % (24 * 3600)) / 3600;
    let minutes = (duration % 3600) / 60;
    let seconds = duration % 60;

    match duration {
        0..=59 => format!("{seconds}s"),
        60..=3599 => format!("{minutes}m"),
        3600..=86_399 => format!("{hours}h {minutes:02}m"),
        _ => format!("{days}d {hours:02}h {minutes:02}m"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // TODO: This is poor test... should make it deterministic
    fn test_convert_datetimeoffset_to_human_readable_remaining_time() {
        let now = OffsetDateTime::now_utc();
        let dt = now + time::Duration::seconds(60);
        assert_eq!(
            convert_datetimeoffset_to_human_readable_remaining_time(dt),
            "1m"
        );
        let dt = now + time::Duration::seconds(3600);
        assert_eq!(
            convert_datetimeoffset_to_human_readable_remaining_time(dt),
            "1h 00m"
        );
    }
}
