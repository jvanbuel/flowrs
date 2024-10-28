use std::collections::HashMap;

use crossterm::event::KeyCode;
use log::debug;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Styled, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table};
use ratatui::Frame;
use time::format_description;

use crate::airflow::model::dag::Dag;
use crate::airflow::model::dagstats::DagStatistic;
use crate::app::events::custom::FlowrsEvent;
use crate::ui::constants::DEFAULT_STYLE;
use crate::ui::TIME_FORMAT;

use super::{filter::Filter, Model, StatefulTable};
use crate::app::error::FlowrsError;
use crate::app::model::popup::PopUp;
use crate::app::worker::WorkerMessage;
use tokio::sync::mpsc::Sender;

pub struct DagModel {
    pub all: Vec<Dag>,
    pub dag_stats: HashMap<String, Vec<DagStatistic>>,
    pub filtered: StatefulTable<Dag>,
    pub filter: Filter,
    #[allow(dead_code)]
    pub popup: PopUp,
    pub errors: Vec<FlowrsError>,
    tx_worker: Option<Sender<WorkerMessage>>,
    ticks: u32,
}

impl DagModel {
    pub fn new() -> Self {
        DagModel {
            all: vec![],
            dag_stats: HashMap::new(),
            filtered: StatefulTable::new(vec![]),
            filter: Filter::new(),
            popup: PopUp::new(),
            errors: vec![],
            ticks: 0,
            tx_worker: None,
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

    pub(crate) fn register_worker(&mut self, tx_worker: Sender<WorkerMessage>) {
        self.tx_worker = Some(tx_worker);
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
    async fn update(&mut self, event: &FlowrsEvent) -> Option<FlowrsEvent> {
        debug!("DagModel::update");
        match event {
            FlowrsEvent::Tick => {
                self.ticks += 1;
                if self.ticks % 10 != 0 {
                    return Some(FlowrsEvent::Tick);
                }
                if let Some(tx_worker) = &self.tx_worker {
                    let _ = tx_worker
                        .send(crate::app::worker::WorkerMessage::UpdateDags)
                        .await;
                    let _ = tx_worker
                        .send(crate::app::worker::WorkerMessage::UpdateDagStats { clear: true })
                        .await;
                }
                Some(FlowrsEvent::Tick)
            }
            FlowrsEvent::Key(key_event) => {
                if self.filter.is_enabled() {
                    self.filter.update(key_event);
                    self.filter_dags();
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
                        KeyCode::Char('G') => {
                            self.filtered.state.select_last();
                            None
                        }
                        KeyCode::Char('t') => {
                            self.popup.is_open = true;
                            None
                        }
                        KeyCode::Char('p') => {
                            let send_channel = self.tx_worker.clone().unwrap();
                            match self.current() {
                                Some(dag) => {
                                    let _ = send_channel
                                        .send(WorkerMessage::ToggleDag {
                                            dag_id: dag.dag_id.clone(),
                                            is_paused: dag.is_paused,
                                        })
                                        .await;
                                    dag.is_paused = !dag.is_paused;
                                }

                                None => self
                                    .errors
                                    .push(FlowrsError::from(String::from("No dag selected"))),
                            }
                            None
                        }
                        KeyCode::Char('/') => {
                            self.filter.toggle();
                            self.filter_dags();
                            None
                        }
                        KeyCode::Enter => {
                            let selected_dag = self.current().map(|dag| dag.dag_id.clone())?;
                            debug!("Selected dag: {}", selected_dag);
                            if let Some(tx_worker) = &self.tx_worker {
                                let _ = tx_worker
                                    .send(WorkerMessage::UpdateDagRuns {
                                        dag_id: selected_dag,
                                        clear: true,
                                    })
                                    .await;
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
        let rects = if self.filter.is_enabled() {
            let rects = Layout::default()
                .constraints([Constraint::Fill(90), Constraint::Max(3)].as_ref())
                .margin(0)
                .split(f.area());

            let filter = self.filter.prefix().clone();

            let paragraph = Paragraph::new(filter.unwrap_or("".to_string()))
                .block(Block::default().borders(Borders::ALL).title("filter"))
                .set_style(DEFAULT_STYLE);
            f.render_widget(paragraph, rects[1]);

            rects
        } else {
            Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .margin(0)
                .split(f.area())
        };

        let selected_style = Style::default().add_modifier(Modifier::REVERSED);

        let headers = ["Active", "Name", "Owners", "Schedule", "Next Run", "Stats"];
        let header_cells = headers.iter().map(|h| Cell::from(*h));
        let header = Row::new(header_cells)
            .style(DEFAULT_STYLE.reversed())
            .add_modifier(Modifier::BOLD);
        let rows = self.filtered.items.iter().enumerate().map(|(idx, item)| {
            Row::new(vec![
                if item.is_paused {
                    Line::from(Span::styled("🔘", Style::default().fg(Color::Gray)))
                } else {
                    Line::from(Span::styled("🔵", Style::default().fg(Color::Gray)))
                },
                Line::from(Span::styled(
                    item.dag_id.as_str(),
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from(item.owners.join(", ")),
                if let Some(schedule) = &item.schedule_interval {
                    Line::from(Span::styled(
                        schedule.value.clone().unwrap_or_else(|| "None".to_string()),
                        Style::default().fg(Color::LightYellow),
                    ))
                } else {
                    Line::from(Span::styled(
                        "None",
                        Style::default().fg(Color::LightYellow),
                    ))
                },
                Line::from(if let Some(date) = item.next_dagrun {
                    date.format(&format_description::parse(TIME_FORMAT).unwrap())
                        .unwrap()
                        .to_string()
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
                                        "success" => Style::default().fg(Color::Green),
                                        "running" if stat.count > 0 => {
                                            Style::default().fg(Color::LightGreen)
                                        }
                                        "failed" if stat.count > 0 => {
                                            Style::default().fg(Color::Red)
                                        }
                                        "queued" => Style::default().fg(Color::LightBlue),
                                        "up_for_retry" => Style::default().fg(Color::LightYellow),
                                        "upstream_failed" => {
                                            Style::default().fg(Color::Rgb(255, 165, 0))
                                        }
                                        _ => Style::default().fg(Color::DarkGray),
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
                DEFAULT_STYLE.bg(Color::Rgb(33, 34, 35))
            })
        });
        let t = Table::new(
            rows,
            &[
                Constraint::Length(7),
                Constraint::Max(60),
                Constraint::Max(15),
                Constraint::Length(10),
                Constraint::Length(20),
                Constraint::Length(30),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("DAGs - Press <p> to toggle pause")
                .border_style(DEFAULT_STYLE)
                .style(DEFAULT_STYLE),
        )
        .row_highlight_style(selected_style);
        f.render_stateful_widget(t, rects[0], &mut self.filtered.state);

        if self.popup.is_open {
            let block = Block::bordered().title("Popup");
            let area = popup_area(rects[0], 60, 20);
            f.render_widget(Clear, area); //this clears out the background
            f.render_widget(block, area);
        }
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
