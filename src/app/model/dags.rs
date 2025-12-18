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
use crate::app::model::popup::dags::commands::DAG_COMMAND_POP_UP;
use crate::ui::common::create_headers;
use crate::ui::constants::AirflowStateColor;
use crate::ui::theme::{
    ACCENT, ALT_ROW_STYLE, BORDER_STYLE, DAG_ACTIVE, DEFAULT_STYLE, SELECTED_ROW_STYLE,
    TABLE_HEADER_STYLE, TEXT_PRIMARY,
};

use super::popup::commands_help::CommandPopUp;
use super::popup::error::ErrorPopup;
use super::{filter::Filter, Model, StatefulTable};
use crate::app::worker::{OpenItem, WorkerMessage};

#[derive(Default)]
pub struct DagModel {
    pub all: Vec<Dag>,
    pub dag_stats: HashMap<String, Vec<DagStatistic>>,
    pub filtered: StatefulTable<Dag>,
    pub filter: Filter,
    commands: Option<&'static CommandPopUp<'static>>,
    pub error_popup: Option<ErrorPopup>,
    ticks: u32,
    event_buffer: Vec<FlowrsEvent>,
}

impl DagModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn filter_dags(&mut self) {
        let prefix = &self.filter.prefix;
        let filtered_dags = match prefix {
            Some(prefix) => &self
                .all
                .iter()
                .filter(|dag| dag.dag_id.contains(prefix))
                .cloned()
                .collect::<Vec<Dag>>(),
            None => &self.all,
        };
        self.filtered.items = filtered_dags.clone();
    }

    pub fn current(&mut self) -> Option<&mut Dag> {
        self.filtered
            .state
            .selected()
            .map(|i| &mut self.filtered.items[i])
    }
    pub fn get_dag_by_id(&self, dag_id: &str) -> Option<&Dag> {
        self.all.iter().find(|dag| dag.dag_id == dag_id)
    }
}

impl Model for DagModel {
    fn update(&mut self, event: &FlowrsEvent) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        match event {
            FlowrsEvent::Tick => {
                self.ticks += 1;
                if !self.ticks.is_multiple_of(10) {
                    return (Some(FlowrsEvent::Tick), vec![]);
                }
                (
                    Some(FlowrsEvent::Tick),
                    vec![
                        WorkerMessage::UpdateDags,
                        WorkerMessage::UpdateDagStats { clear: true },
                    ],
                )
            }
            FlowrsEvent::Key(key_event) => {
                if self.filter.is_enabled() {
                    self.filter.update(key_event);
                    self.filter_dags();
                    return (None, vec![]);
                } else if let Some(_error_popup) = &mut self.error_popup {
                    match key_event.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            self.error_popup = None;
                        }
                        _ => (),
                    }
                    return (None, vec![]);
                } else if let Some(_commands) = &mut self.commands {
                    match key_event.code {
                        KeyCode::Char('q' | '?') | KeyCode::Esc => {
                            self.commands = None;
                        }
                        _ => (),
                    }
                } else {
                    match key_event.code {
                        KeyCode::Down | KeyCode::Char('j') => {
                            self.filtered.next();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.filtered.previous();
                        }
                        KeyCode::Char('G') => {
                            if !self.filtered.items.is_empty() {
                                self.filtered
                                    .state
                                    .select(Some(self.filtered.items.len() - 1));
                            }
                        }
                        KeyCode::Char('p') => match self.current() {
                            Some(dag) => {
                                let current_state = dag.is_paused;
                                dag.is_paused = !current_state;
                                return (
                                    None,
                                    vec![WorkerMessage::ToggleDag {
                                        dag_id: dag.dag_id.clone(),
                                        is_paused: current_state,
                                    }],
                                );
                            }
                            None => {
                                self.error_popup = Some(ErrorPopup::from_strings(vec![
                                    "No DAG selected to pause/resume".to_string(),
                                ]));
                            }
                        },
                        KeyCode::Char('/') => {
                            self.filter.toggle();
                            self.filter_dags();
                        }
                        KeyCode::Char('?') => {
                            self.commands = Some(&*DAG_COMMAND_POP_UP);
                        }
                        KeyCode::Enter => {
                            if let Some(selected_dag) = self.current().map(|dag| dag.dag_id.clone())
                            {
                                debug!("Selected dag: {selected_dag}");
                                return (
                                    Some(FlowrsEvent::Key(*key_event)),
                                    vec![WorkerMessage::UpdateDagRuns {
                                        dag_id: selected_dag,
                                        clear: true,
                                    }],
                                );
                            }
                            self.error_popup = Some(ErrorPopup::from_strings(vec![
                                "No DAG selected to view DAG Runs".to_string(),
                            ]));
                        }
                        KeyCode::Char('g') => {
                            if let Some(FlowrsEvent::Key(key_event)) = self.event_buffer.pop() {
                                if key_event.code == KeyCode::Char('g') {
                                    self.filtered.state.select_first();
                                } else {
                                    self.event_buffer.push(FlowrsEvent::Key(key_event));
                                }
                            } else {
                                self.event_buffer.push(FlowrsEvent::Key(*key_event));
                            }
                        }
                        KeyCode::Char('o') => {
                            if let Some(dag) = self.current() {
                                debug!("Selected dag: {}", dag.dag_id);
                                return (
                                    Some(FlowrsEvent::Key(*key_event)),
                                    vec![WorkerMessage::OpenItem(OpenItem::Dag {
                                        dag_id: dag.dag_id.clone(),
                                    })],
                                );
                            }
                            self.error_popup = Some(ErrorPopup::from_strings(vec![
                                "No DAG selected to open in the browser".to_string(),
                            ]));
                        }
                        _ => return (Some(FlowrsEvent::Key(*key_event)), vec![]), // if no match, return the event
                    }
                    return (None, vec![]);
                }
                (None, vec![])
            }
            FlowrsEvent::Mouse | FlowrsEvent::FocusGained | FlowrsEvent::FocusLost => {
                (Some(event.clone()), vec![])
            }
        }
    }
}

impl Widget for &mut DagModel {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rects = if self.filter.is_enabled() {
            let rects = Layout::default()
                .constraints([Constraint::Fill(90), Constraint::Max(3)].as_ref())
                .margin(0)
                .split(area);

            self.filter.render(rects[1], buf);

            rects
        } else {
            Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .margin(0)
                .split(area)
        };
        let headers = ["Active", "Name", "Owners", "Schedule", "Next Run", "Stats"];
        let header_row = create_headers(headers);
        let header = Row::new(header_row).style(TABLE_HEADER_STYLE);
        let rows =
            self.filtered.items.iter().enumerate().map(|(idx, item)| {
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
                    Line::from(
                        item.next_dagrun_create_after.map_or_else(|| "None".to_string(), convert_datetimeoffset_to_human_readable_remaining_time),
                    ),
                    Line::from(self.dag_stats.get(&item.dag_id).map_or_else(
                        || vec![Span::styled("None".to_string(), Style::default())],
                        |stats| {
                            stats
                                .iter()
                                .map(|stat| {
                                    Span::styled(
                                        left_pad::leftpad(stat.count.to_string(), 7),
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
                                            _ => {
                                                Style::default().fg(AirflowStateColor::None.into())
                                            }
                                        },
                                    )
                                })
                                .collect::<Vec<Span>>()
                        },
                    )),
                ])
                .style(if (idx % 2) == 0 {
                    DEFAULT_STYLE
                } else {
                    ALT_ROW_STYLE
                })
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
            if let Some(prefix) = self.filter.prefix() {
                block.title_bottom(Line::from(Span::styled(
                    format!(" Filter: {prefix} "),
                    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                )))
            } else {
                block
            }
        })
        .row_highlight_style(SELECTED_ROW_STYLE);

        StatefulWidget::render(t, rects[0], buf, &mut self.filtered.state);

        if let Some(commands) = &self.commands {
            commands.render(area, buf);
        }

        if let Some(error_popup) = &self.error_popup {
            error_popup.render(area, buf);
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
