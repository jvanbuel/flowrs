use std::collections::HashMap;

use crossterm::event::KeyCode;
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Row, StatefulWidget, Table, Widget};
use time::OffsetDateTime;

use crate::airflow::model::dag::Dag;
use crate::airflow::model::dagstats::DagStatistic;
use crate::app::events::custom::FlowrsEvent;
use crate::app::model::popup::dags::commands::DAG_COMMAND_POP_UP;
use crate::ui::common::create_headers;
use crate::ui::constants::{AirflowStateColor, ALTERNATING_ROW_COLOR, DEFAULT_STYLE};

use super::popup::commands_help::CommandPopUp;
use super::{filter::Filter, Model, StatefulTable};
use crate::app::worker::{OpenItem, WorkerMessage};
use anyhow::Error;

pub struct DagModel {
    pub all: Vec<Dag>,
    pub dag_stats: HashMap<String, Vec<DagStatistic>>,
    pub filtered: StatefulTable<Dag>,
    pub filter: Filter,
    #[allow(dead_code)]
    pub errors: Vec<Error>,
    commands: Option<&'static CommandPopUp<'static>>,
    ticks: u32,
    event_buffer: Vec<FlowrsEvent>,
}

impl DagModel {
    pub fn new() -> Self {
        DagModel {
            all: vec![],
            dag_stats: HashMap::new(),
            filtered: StatefulTable::new(vec![]),
            filter: Filter::new(),
            errors: vec![],
            ticks: 0,
            commands: None,
            event_buffer: vec![],
        }
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
        self.filtered.items = filtered_dags.to_vec();
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

impl Default for DagModel {
    fn default() -> Self {
        Self::new()
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
                } else if let Some(_commands) = &mut self.commands {
                    match key_event.code {
                        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('?') => {
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
                            self.filtered.state.select_last();
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
                            None => self
                                .errors
                                .push(Error::msg("No DAG selected to pause/resume")),
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
                                debug!("Selected dag: {}", selected_dag);
                                return (
                                    Some(FlowrsEvent::Key(*key_event)),
                                    vec![WorkerMessage::UpdateDagRuns {
                                        dag_id: selected_dag,
                                        clear: true,
                                    }],
                                );
                            } else {
                                self.errors
                                    .push(Error::msg("No DAG selected to view DAG Runs"));
                            }
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
                            } else {
                                self.errors
                                    .push(Error::msg("No DAG selected to open in the browser"));
                            }
                        }
                        _ => return (Some(FlowrsEvent::Key(*key_event)), vec![]), // if no match, return the event
                    }
                    return (None, vec![]);
                }
                (None, vec![])
            }
            _ => (Some(event.clone()), vec![]),
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
        let selected_style = DEFAULT_STYLE.add_modifier(Modifier::REVERSED);

        let headers = ["Active", "Name", "Owners", "Schedule", "Next Run", "Stats"];
        let header_row = create_headers(headers);
        let header = Row::new(header_row)
            .style(DEFAULT_STYLE.reversed())
            .add_modifier(Modifier::BOLD);
        let rows =
            self.filtered.items.iter().enumerate().map(|(idx, item)| {
                Row::new(vec![
                    if item.is_paused {
                        Line::from(Span::styled("🔘", Style::default()))
                    } else {
                        Line::from(Span::styled("🔵", Style::default()))
                    },
                    Line::from(Span::styled(
                        item.dag_id.as_str(),
                        Style::default().add_modifier(Modifier::BOLD),
                    )),
                    Line::from(item.owners.join(", ")),
                    if let Some(schedule) = &item.schedule_interval {
                        Line::from(schedule.value.clone().unwrap_or_else(|| "None".to_string()))
                    } else {
                        Line::from("None")
                    }
                    .style(Style::default().fg(Color::LightYellow)),
                    Line::from(if let Some(date) = item.next_dagrun_create_after {
                        convert_datetimeoffset_to_human_readable_remaining_time(date)
                    } else {
                        "None".to_string()
                    }),
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
                    DEFAULT_STYLE.bg(ALTERNATING_ROW_COLOR)
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
        .block(
            Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .title("DAGs - Press <?> to see available commands")
                .border_style(DEFAULT_STYLE)
                .style(DEFAULT_STYLE),
        )
        .row_highlight_style(selected_style);

        StatefulWidget::render(t, rects[0], buf, &mut self.filtered.state);

        if let Some(commands) = &self.commands {
            commands.render(area, buf);
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
    let duration = if duration < 0 { 0 } else { duration as u64 };
    let days = duration / (24 * 3600);
    let hours = (duration % (24 * 3600)) / 3600;
    let minutes = (duration % 3600) / 60;
    let seconds = duration % 60;

    match duration {
        0..=59 => format!("{}s", seconds),
        60..=3599 => format!("{}m", minutes),
        3600..=86_399 => format!("{}h {:02}m", hours, minutes),
        _ => format!("{}d {:02}h {:02}m", days, hours, minutes),
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
