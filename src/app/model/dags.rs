use std::collections::HashMap;

use crossterm::event::KeyCode;
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Row, StatefulWidget, Table, Widget};
use time::OffsetDateTime;

use crate::airflow::model::common::{Dag, DagStatistic};
use crate::app::events::custom::FlowrsEvent;
use crate::app::model::popup::dagruns::trigger::TriggerDagRunPopUp;
use crate::app::model::popup::dags::commands::DAG_COMMAND_POP_UP;
use crate::ui::common::create_headers;
use crate::ui::constants::AirflowStateColor;
use crate::ui::theme::{
    BORDER_STYLE, DAG_ACTIVE, SELECTED_ROW_STYLE, TABLE_HEADER_STYLE, TEXT_PRIMARY,
};

use super::popup::dags::DagPopUp;
use super::{FilterableTable, KeyResult, Model, Popup};
use crate::app::worker::{OpenItem, WorkerMessage};

/// Model for the DAG panel, managing the list of DAGs and their filtering.
pub struct DagModel {
    /// Filterable table containing all DAGs and filtered view
    pub table: FilterableTable<Dag>,
    /// DAG statistics by `dag_id`
    pub dag_stats: HashMap<String, Vec<DagStatistic>>,
    /// Unified popup state (error, commands, or custom for this model)
    pub popup: Popup<DagPopUp>,
    ticks: u32,
    event_buffer: Vec<KeyCode>,
}

impl Default for DagModel {
    fn default() -> Self {
        Self {
            table: FilterableTable::new(),
            dag_stats: HashMap::new(),
            popup: Popup::None,
            ticks: 0,
            event_buffer: Vec::new(),
        }
    }
}

impl DagModel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Find a DAG by its ID
    pub fn get_dag_by_id(&self, dag_id: &str) -> Option<&Dag> {
        self.table.all.iter().find(|dag| dag.dag_id == dag_id)
    }
}

impl DagModel {
    /// Handle model-specific popup (returns messages from popup)
    fn handle_popup(
        &mut self,
        event: &FlowrsEvent,
        ctx: &crate::app::state::NavigationContext,
    ) -> Option<Vec<WorkerMessage>> {
        let custom_popup = self.popup.custom_mut()?;
        let DagPopUp::Trigger(trigger_popup) = custom_popup;
        let (key_event, messages) = trigger_popup.update(event, ctx);
        debug!("Popup messages: {messages:?}");

        if let Some(FlowrsEvent::Key(key_event)) = &key_event {
            if matches!(
                key_event.code,
                KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q')
            ) {
                self.popup.close();
            }
        }
        Some(messages)
    }

    /// Handle model-specific keys
    fn handle_keys(&mut self, key_code: KeyCode) -> KeyResult {
        match key_code {
            KeyCode::Char('p') => {
                if let Some(dag) = self.table.current_mut() {
                    let current_state = dag.is_paused;
                    dag.is_paused = !current_state;
                    KeyResult::ConsumedWith(vec![WorkerMessage::ToggleDag {
                        dag_id: dag.dag_id.clone(),
                        is_paused: current_state,
                    }])
                } else {
                    self.popup
                        .show_error(vec!["No DAG selected to pause/resume".to_string()]);
                    KeyResult::Consumed
                }
            }
            KeyCode::Char('?') => {
                self.popup.show_commands(&DAG_COMMAND_POP_UP);
                KeyResult::Consumed
            }
            KeyCode::Enter => {
                if let Some(dag) = self.table.current() {
                    debug!("Selected dag: {}", dag.dag_id);
                    KeyResult::PassWith(vec![WorkerMessage::UpdateDagRuns {
                        dag_id: dag.dag_id.clone(),
                    }])
                } else {
                    self.popup
                        .show_error(vec!["No DAG selected to view DAG Runs".to_string()]);
                    KeyResult::Consumed
                }
            }
            KeyCode::Char('o') => {
                if let Some(dag) = self.table.current() {
                    debug!("Selected dag: {}", dag.dag_id);
                    KeyResult::PassWith(vec![WorkerMessage::OpenItem(OpenItem::Dag {
                        dag_id: dag.dag_id.clone(),
                    })])
                } else {
                    self.popup
                        .show_error(vec!["No DAG selected to open in the browser".to_string()]);
                    KeyResult::Consumed
                }
            }
            KeyCode::Char('t') => {
                if let Some(dag) = self.table.current() {
                    self.popup
                        .show_custom(DagPopUp::Trigger(TriggerDagRunPopUp::new(
                            dag.dag_id.clone(),
                        )));
                } else {
                    self.popup
                        .show_error(vec!["No DAG selected to trigger".to_string()]);
                }
                KeyResult::Consumed
            }
            _ => KeyResult::PassThrough,
        }
    }
}

impl Model for DagModel {
    fn update(
        &mut self,
        event: &FlowrsEvent,
        ctx: &crate::app::state::NavigationContext,
    ) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        match event {
            FlowrsEvent::Tick => {
                self.ticks += 1;
                if !self.ticks.is_multiple_of(10) {
                    return (Some(FlowrsEvent::Tick), vec![]);
                }
                (
                    Some(FlowrsEvent::Tick),
                    vec![WorkerMessage::UpdateDagsAndStats],
                )
            }
            FlowrsEvent::Key(key_event) => {
                // Popup handling (has its own update method)
                if let Some(messages) = self.handle_popup(event, ctx) {
                    return (None, messages);
                }

                // Chain the handlers
                let result = self
                    .table
                    .handle_filter_key(key_event)
                    .or_else(|| self.popup.handle_dismiss(key_event.code))
                    .or_else(|| {
                        self.table
                            .handle_navigation(key_event.code, &mut self.event_buffer)
                    })
                    .or_else(|| self.handle_keys(key_event.code));

                result.into_result(event)
            }
            FlowrsEvent::Mouse | FlowrsEvent::FocusGained | FlowrsEvent::FocusLost => {
                (Some(event.clone()), vec![])
            }
        }
    }
}

impl Widget for &mut DagModel {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let content_area = self.table.render_with_filter(area, buf);

        let headers = ["Active", "Name", "Owners", "Schedule", "Next Run", "Stats"];
        let header_row = create_headers(headers);
        let header = Row::new(header_row).style(TABLE_HEADER_STYLE);
        let rows =
            self.table
                .filtered
                .items
                .iter()
                .enumerate()
                .map(|(idx, item)| {
                    Row::new(vec![
                        if item.is_paused {
                            Line::from(Span::styled("𖣘", Style::default().fg(TEXT_PRIMARY)))
                        } else {
                            Line::from(Span::styled("𖣘", Style::default().fg(DAG_ACTIVE)))
                        },
                        Line::from(Span::styled(
                            item.dag_id.as_str(),
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
                                            match stat.state.as_str() {
                                                "success" => Style::default()
                                                    .fg(AirflowStateColor::Success.into()),
                                                "running" if stat.count > 0 => Style::default()
                                                    .fg(AirflowStateColor::Running.into()),
                                                "failed" if stat.count > 0 => Style::default()
                                                    .fg(AirflowStateColor::Failed.into()),
                                                "queued" => Style::default()
                                                    .fg(AirflowStateColor::Queued.into()),
                                                "up_for_retry" => Style::default()
                                                    .fg(AirflowStateColor::UpForRetry.into()),
                                                "upstream_failed" => Style::default()
                                                    .fg(AirflowStateColor::UpstreamFailed.into()),
                                                _ => Style::default()
                                                    .fg(AirflowStateColor::None.into()),
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

        // Render any active popup (error, commands, or custom)
        (&self.popup).render(area, buf);

        // Render custom popups that need special handling
        if let Some(DagPopUp::Trigger(trigger_popup)) = self.popup.custom_mut() {
            trigger_popup.render(area, buf);
        }
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
#[allow(dead_code)]
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
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
