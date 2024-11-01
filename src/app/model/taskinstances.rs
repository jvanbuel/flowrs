use std::vec;

use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use ratatui::Frame;
use time::format_description;

use crate::airflow::model::taskinstance::TaskInstance;
use crate::app::events::custom::FlowrsEvent;
use crate::ui::constants::DEFAULT_STYLE;
use crate::ui::TIME_FORMAT;

use super::{filter::Filter, Model, StatefulTable};
use crate::app::error::FlowrsError;
use crate::app::worker::WorkerMessage;

pub struct TaskInstanceModel {
    pub dag_id: Option<String>,
    pub dag_run_id: Option<String>,
    pub all: Vec<TaskInstance>,
    pub filtered: StatefulTable<TaskInstance>,
    pub filter: Filter,
    #[allow(dead_code)]
    pub errors: Vec<FlowrsError>,
    ticks: u32,
}

impl TaskInstanceModel {
    pub fn new() -> Self {
        TaskInstanceModel {
            dag_id: None,
            dag_run_id: None,
            all: vec![],
            filtered: StatefulTable::new(vec![]),
            filter: Filter::new(),
            errors: vec![],
            ticks: 0,
        }
    }

    pub fn filter_task_instances(&mut self) {
        let prefix = &self.filter.prefix;
        let filtered_task_instances = match prefix {
            Some(prefix) => &self
                .all
                .iter()
                .filter(|dagrun| dagrun.dag_run_id.contains(prefix))
                .cloned()
                .collect::<Vec<TaskInstance>>(),
            None => &self.all,
        };
        self.filtered.items = filtered_task_instances.to_vec();
    }

    #[allow(dead_code)]
    pub fn current(&mut self) -> Option<&mut TaskInstance> {
        self.filtered
            .state
            .selected()
            .map(|i| &mut self.filtered.items[i])
    }
}

impl Model for TaskInstanceModel {
    fn update(&mut self, event: &FlowrsEvent) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        match event {
            FlowrsEvent::Tick => {
                self.ticks += 1;
                if self.ticks % 10 != 0 {
                    return (Some(FlowrsEvent::Tick), vec![]);
                }
                if let (Some(dag_run_id), Some(dag_id)) = (&self.dag_run_id, &self.dag_id) {
                    log::debug!("Updating task instances for dag_run_id: {}", dag_run_id);
                    return (
                        Some(FlowrsEvent::Tick),
                        vec![WorkerMessage::UpdateTaskInstances {
                            dag_id: dag_id.clone(),
                            dag_run_id: dag_run_id.clone(),
                            clear: false,
                        }],
                    );
                }
                (Some(FlowrsEvent::Tick), vec![])
            }
            FlowrsEvent::Key(key_event) => {
                if self.filter.is_enabled() {
                    self.filter.update(key_event);
                    self.filter_task_instances();
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
                        KeyCode::Char('/') => {
                            self.filter.toggle();
                            self.filter_task_instances();
                        }
                        _ => return (Some(FlowrsEvent::Key(*key_event)), vec![]), // if no match, return the event
                    }
                }
                (None, vec![])
            }
            _ => (Some(event.clone()), vec![]),
        }
    }

    fn view(&mut self, f: &mut Frame) {
        let rects = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .margin(0)
            .split(f.area());

        let selected_style = Style::default().add_modifier(Modifier::REVERSED);

        let headers = ["Task ID", "Execution Date", "Duration", "State", "Tries"];
        let header_cells = headers.iter().map(|h| Cell::from(*h));
        let header =
            Row::new(header_cells).style(DEFAULT_STYLE.reversed().add_modifier(Modifier::BOLD));

        let rows = self.filtered.items.iter().enumerate().map(|(idx, item)| {
            Row::new(vec![
                Line::from(item.task_id.as_str()),
                Line::from(if let Some(date) = item.execution_date {
                    date.format(&format_description::parse(TIME_FORMAT).unwrap())
                        .unwrap()
                        .to_string()
                } else {
                    "None".to_string()
                }),
                Line::from(if let Some(i) = item.duration {
                    format!("{i}")
                } else {
                    "None".to_string()
                }),
                Line::from(match item.state.as_str() {
                    "success" => state_to_color(item, Color::Green),
                    "running" => state_to_color(item, Color::LightGreen),
                    "failed" => state_to_color(item, Color::Red),
                    "queued" => state_to_color(item, Color::LightBlue),
                    "up_for_retry" => state_to_color(item, Color::LightYellow),
                    "upstream_failed" => state_to_color(item, Color::Rgb(255, 165, 0)),
                    _ => state_to_color(item, Color::White),
                }),
                Line::from(format!("{:?}", item.try_number)),
            ])
            .style(if (idx % 2) == 0 {
                DEFAULT_STYLE
            } else {
                DEFAULT_STYLE.bg(Color::Rgb(33, 34, 35))
            })
        });
        let t = Table::new(
            rows,
            &[
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Length(20),
                Constraint::Length(15),
                Constraint::Length(5),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("TaskInstances"),
        )
        .style(DEFAULT_STYLE)
        .row_highlight_style(selected_style);

        f.render_stateful_widget(t, rects[0], &mut self.filtered.state);
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

fn state_to_color(item: &TaskInstance, color: Color) -> Span {
    Span::styled(item.state.as_str(), Style::default().fg(color))
}
