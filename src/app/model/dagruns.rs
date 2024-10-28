use crossterm::event::KeyCode;
use log::debug;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use ratatui::Frame;
use time::format_description;

use crate::airflow::model::dagrun::DagRun;
use crate::app::events::custom::FlowrsEvent;
use crate::ui::constants::DEFAULT_STYLE;
use crate::ui::TIME_FORMAT;

use super::{filter::Filter, Model, StatefulTable};
use crate::app::error::FlowrsError;
use crate::app::model::popup::PopUp;
use crate::app::worker::WorkerMessage;
use tokio::sync::mpsc::Sender;

pub struct DagRunModel {
    pub dag_id: Option<String>,
    pub dag_code: Option<String>,
    pub all: Vec<DagRun>,
    pub filtered: StatefulTable<DagRun>,
    pub filter: Filter,
    #[allow(dead_code)]
    pub popup: PopUp,
    pub errors: Vec<FlowrsError>,
    tx_worker: Option<Sender<WorkerMessage>>,
    ticks: u32,
}

impl DagRunModel {
    pub fn new() -> Self {
        DagRunModel {
            dag_id: None,
            dag_code: None,
            all: vec![],
            filtered: StatefulTable::new(vec![]),
            filter: Filter::new(),
            popup: PopUp::new(),
            errors: vec![],
            ticks: 0,
            tx_worker: None,
        }
    }

    pub fn filter_dag_runs(&mut self) {
        let prefix = &self.filter.prefix;
        let filtered_dag_runs = match prefix {
            Some(prefix) => &self
                .all
                .iter()
                .filter(|dagrun| dagrun.dag_run_id.contains(prefix))
                .cloned()
                .collect::<Vec<DagRun>>(),
            None => &self.all,
        };
        self.filtered.items = filtered_dag_runs.to_vec();
    }

    pub(crate) fn register_worker(&mut self, tx_worker: Sender<WorkerMessage>) {
        self.tx_worker = Some(tx_worker);
    }

    #[allow(dead_code)]
    pub fn current(&self) -> Option<&DagRun> {
        self.filtered
            .state
            .selected()
            .map(|i| &self.filtered.items[i])
    }
}

impl Model for DagRunModel {
    async fn update(&mut self, event: &FlowrsEvent) -> Option<FlowrsEvent> {
        debug!("DagRunModel::update");
        match event {
            FlowrsEvent::Tick => {
                self.ticks += 1;
                if self.ticks % 10 != 0 {
                    return Some(FlowrsEvent::Tick);
                }
                if let Some(dag_id) = &self.dag_id {
                    log::debug!("Updating dagruns for dag_id: {}", dag_id);
                    if let Some(tx_worker) = &self.tx_worker {
                        let _ = tx_worker
                            .send(crate::app::worker::WorkerMessage::UpdateDagRuns {
                                dag_id: dag_id.clone(),
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
                            self.filter_dag_runs();
                            None
                        }
                        KeyCode::Char('v') => {
                            if let Some(dag_id) = &self.dag_id {
                                if let Some(tx_worker) = &self.tx_worker {
                                    let _ = tx_worker
                                        .send(WorkerMessage::GetDagCode {
                                            dag_id: dag_id.clone(),
                                        })
                                        .await;
                                }
                            }
                            None
                        }
                        KeyCode::Enter => {
                            if let (Some(dag_id), Some(dag_run)) = (&self.dag_id, &self.current()) {
                                if let Some(tx_worker) = &self.tx_worker {
                                    let _ = tx_worker
                                        .send(WorkerMessage::UpdateTaskInstances {
                                            dag_id: dag_id.clone(),
                                            dag_run_id: dag_run.dag_run_id.clone(),
                                            clear: true,
                                        })
                                        .await;
                                }
                            }
                            Some(FlowrsEvent::Key(*key_event))
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

        let normal_style = DEFAULT_STYLE;

        let headers = ["State", "DAG Run ID", "Logical Date", "Type"];
        let header_cells = headers.iter().map(|h| Cell::from(*h).style(normal_style));
        let header =
            Row::new(header_cells).style(DEFAULT_STYLE.reversed().add_modifier(Modifier::BOLD));

        let rows = self.filtered.items.iter().enumerate().map(|(idx, item)| {
            Row::new(vec![
                Line::from(match item.state.as_str() {
                    "success" => Span::styled("■", Style::default().fg(Color::Rgb(0, 128, 0))),
                    "running" => Span::styled("■", DEFAULT_STYLE.fg(Color::LightGreen)),
                    "failed" => Span::styled("■", DEFAULT_STYLE.fg(Color::Red)),
                    "queued" => Span::styled("■", DEFAULT_STYLE.fg(Color::LightBlue)),
                    _ => Span::styled("■", DEFAULT_STYLE.fg(Color::White)),
                }),
                Line::from(Span::styled(
                    item.dag_run_id.as_str(),
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from(if let Some(date) = item.logical_date {
                    date.format(&format_description::parse(TIME_FORMAT).unwrap())
                        .unwrap()
                        .to_string()
                } else {
                    "None".to_string()
                }),
                Line::from(item.run_type.as_str()),
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
                Constraint::Length(7),
                Constraint::Percentage(20),
                Constraint::Max(22),
                Constraint::Length(10),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(if let Some(dag_id) = &self.dag_id {
                    format!("DAGRuns ({})", dag_id)
                } else {
                    "DAGRuns".to_string()
                })
                .style(DEFAULT_STYLE),
        )
        .row_highlight_style(DEFAULT_STYLE.reversed());
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
