use crossterm::event::KeyCode;
use log::debug;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use ratatui::Frame;
use time::format_description;

use crate::airflow::model::taskinstance::TaskInstance;
use crate::app::events::custom::FlowrsEvent;
use crate::ui::constants::{DEFAULT_STYLE, DM_RGB};
use crate::ui::TIME_FORMAT;

use super::{filter::Filter, Model, StatefulTable};
use crate::app::error::FlowrsError;
use crate::app::model::popup::PopUp;
use crate::app::worker::WorkerMessage;
use tokio::sync::mpsc::Sender;

pub struct TaskInstanceModel {
    pub dag_id: Option<String>,
    pub dag_run_id: Option<String>,
    pub all: Vec<TaskInstance>,
    pub filtered: StatefulTable<TaskInstance>,
    pub filter: Filter,
    #[allow(dead_code)]
    pub popup: PopUp,
    pub errors: Vec<FlowrsError>,
    tx_worker: Option<Sender<WorkerMessage>>,
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
            popup: PopUp::new(),
            errors: vec![],
            ticks: 0,
            tx_worker: None,
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

    pub(crate) fn register_worker(&mut self, tx_worker: Sender<WorkerMessage>) {
        self.tx_worker = Some(tx_worker);
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
    async fn update(&mut self, event: &FlowrsEvent) -> Option<FlowrsEvent> {
        debug!("DagRunModel::update");
        match event {
            FlowrsEvent::Tick => {
                self.ticks += 1;
                if self.ticks % 10 != 0 {
                    return Some(FlowrsEvent::Tick);
                }
                if let (Some(dag_run_id), Some(dag_id)) = (&self.dag_run_id, &self.dag_id) {
                    log::debug!("Updating task instances for dag_run_id: {}", dag_run_id);
                    if let Some(tx_worker) = &self.tx_worker {
                        let _ = tx_worker
                            .send(crate::app::worker::WorkerMessage::UpdateTaskInstances {
                                dag_id: dag_id.clone(),
                                dag_run_id: dag_run_id.clone(),
                                clear: false,
                            })
                            .await;
                    }
                }
                Some(FlowrsEvent::Tick)
            }
            FlowrsEvent::Key(key_event) => {
                if self.filter.is_enabled() {
                    self.filter.update(key_event);
                    None
                } else if self.popup.is_open {
                    match key_event.code {
                        KeyCode::Enter => {
                            self.popup.is_open = false;
                        }
                        KeyCode::Esc => {
                            self.popup.is_open = false;
                        }
                        _ => {}
                    }
                    None
                } else {
                    match key_event.code {
                        KeyCode::Down | KeyCode::Char('j') => {
                            self.filtered.next();
                            None
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.filtered.previous();
                            None
                        }
                        KeyCode::Char('t') => {
                            self.popup.is_open = true;
                            None
                        }
                        KeyCode::Char('/') => {
                            self.filter.toggle();
                            self.filter_task_instances();
                            None
                        }
                        _ => Some(FlowrsEvent::Key(*key_event)), // if no match, return the event
                    }
                }
            }
            _ => None,
        }
    }

    fn view(&mut self, f: &mut Frame) {
        let rects = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .margin(0)
            .split(f.area());

        let selected_style = Style::default().add_modifier(Modifier::REVERSED);
        let normal_style = Style::default().bg(DM_RGB);

        let headers = ["Task ID", "Execution Date", "Duration", "State", "Tries"];
        let header_cells = headers.iter().map(|h| Cell::from(*h).style(normal_style));
        let header = Row::new(header_cells).style(normal_style.add_modifier(Modifier::BOLD));
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
                    "success" => {
                        Span::styled(item.state.as_str(), Style::default().fg(Color::Green))
                    }
                    "running" => {
                        Span::styled(item.state.as_str(), Style::default().fg(Color::LightGreen))
                    }
                    "failed" => Span::styled(item.state.as_str(), Style::default().fg(Color::Red)),
                    "queued" => {
                        Span::styled(item.state.as_str(), Style::default().fg(Color::LightBlue))
                    }
                    "up_for_retry" => {
                        Span::styled(item.state.as_str(), Style::default().fg(Color::LightYellow))
                    }
                    "upstream_failed" => Span::styled(
                        item.state.as_str(),
                        Style::default().fg(Color::Rgb(255, 165, 0)), // orange
                    ),
                    _ => Span::styled(item.state.as_str(), Style::default().fg(Color::White)),
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
