use std::sync::{atomic::Ordering::Relaxed, Arc};

use crossterm::event::KeyCode;
use log::debug;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style, Styled, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;
use time::format_description;

use crate::airflow::model::dag::Dag;
use crate::app::events::custom::FlowrsEvent;
use crate::ui::constants::DEFAULT_STYLE;
use crate::ui::TIME_FORMAT;

use super::{filter::Filter, Model, StatefulTable};
use crate::app::error::FlowrsError;
use crate::app::model::popup::PopUp;
use crate::app::state::FlowrsContext;

pub struct DagModel {
    pub all: Vec<Dag>,
    pub filtered: StatefulTable<Dag>,
    pub filter: Filter,
    pub popup: PopUp,
    pub errors: Vec<FlowrsError>,
    pub context: Arc<FlowrsContext>,
}

impl DagModel {
    pub fn new(context: Arc<FlowrsContext>) -> Self {
        DagModel {
            all: vec![],
            filtered: StatefulTable::new(vec![]),
            filter: Filter::new(),
            popup: PopUp::new(),
            errors: vec![],
            context,
        }
    }

    pub async fn update_dags(&mut self) {
        let dag_list = self.context.client.list_dags().await;
        match dag_list {
            Ok(dag_list) => {
                self.all = dag_list.dags;
                self.filter_dags();
            }
            Err(e) => self.errors.push(e),
        }
    }

    fn filter_dags(&mut self) {
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

    pub async fn toggle_current_dag(&mut self) {
        let i = self.filtered.state.selected().unwrap_or(0);
        let dag_id = &self.filtered.items[i].dag_id.clone();
        let is_paused = self.filtered.items[i].is_paused;

        // Don't wait for API response to update UI
        self.filtered.items[i].is_paused = !is_paused;

        match self.context.client.toggle_dag(dag_id, is_paused).await {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error toggling dag: {}", e);
                self.filtered.items[i].is_paused = is_paused;
            }
        }
    }
}

impl Model for DagModel {
    async fn update(&mut self, event: &FlowrsEvent) -> Option<FlowrsEvent> {
        debug!("DagModel::update");
        match event {
            FlowrsEvent::Tick => {
                if self.context.ticks.load(Relaxed) % 10 != 0 {
                    return None;
                }
                self.update_dags().await;
                None
            }
            FlowrsEvent::Key(key_event) => {
                if self.filter.is_enabled() {
                    self.filter.update(key_event);
                    self.filter_dags();
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
                            self.toggle_current_dag().await;
                            None
                        }
                        KeyCode::Char('/') => {
                            self.filter.toggle();
                            self.filter_dags();
                            None
                        }
                        _ => None,
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

        let headers = ["Active", "Name", "Owners", "Schedule", "Next Run"];
        let header_cells = headers.iter().map(|h| Cell::from(*h));
        let header = Row::new(header_cells)
            .style(DEFAULT_STYLE.reversed())
            .add_modifier(Modifier::BOLD);
        // .underlined();
        let rows = self.filtered.items.iter().map(|item| {
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
            ])
            // .height(height as u16)
            // .bottom_margin(1)
            .style(DEFAULT_STYLE)
        });
        let t = Table::new(
            rows,
            &[
                Constraint::Length(7),
                Constraint::Percentage(40),
                Constraint::Max(15),
                Constraint::Length(10),
                Constraint::Fill(1),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("DAGs")
                .border_style(DEFAULT_STYLE)
                .style(DEFAULT_STYLE),
        )
        .highlight_style(selected_style);
        f.render_stateful_widget(t, rects[0], &mut self.filtered.state);
    }
}
