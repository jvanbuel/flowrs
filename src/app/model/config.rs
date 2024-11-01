use crossterm::event::KeyCode;
use log::debug;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use ratatui::Frame;

use crate::airflow::config::AirflowConfig;
use crate::app::events::custom::FlowrsEvent;
use crate::app::worker::WorkerMessage;
use crate::ui::constants::DEFAULT_STYLE;

use super::{filter::Filter, Model, StatefulTable};
use crate::app::error::FlowrsError;

pub struct ConfigModel {
    pub all: Vec<AirflowConfig>,
    pub filtered: StatefulTable<AirflowConfig>,
    pub filter: Filter,
    #[allow(dead_code)]
    #[allow(dead_code)]
    pub errors: Vec<FlowrsError>,
}

impl ConfigModel {
    pub fn new(configs: Vec<AirflowConfig>) -> Self {
        ConfigModel {
            all: configs.clone(),
            filtered: StatefulTable::new(configs),
            filter: Filter::new(),
            errors: vec![],
        }
    }

    pub fn filter_configs(&mut self) {
        let prefix = &self.filter.prefix;
        let dags = &self.all;
        let filtered_configs = match prefix {
            Some(prefix) => self
                .all
                .iter()
                .filter(|config| config.name.contains(prefix))
                .cloned()
                .collect::<Vec<AirflowConfig>>(),
            None => dags.to_vec(),
        };
        self.filtered.items = filtered_configs;
    }
}

impl Model for ConfigModel {
    fn update(&mut self, event: &FlowrsEvent) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        if let FlowrsEvent::Key(key_event) = event {
            if self.filter.enabled {
                self.filter.update(key_event);
                self.filter_configs();
                return (None, vec![]);
            }
            match key_event.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    self.filtered.next();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.filtered.previous();
                }
                KeyCode::Char('/') => {
                    self.filter.toggle();
                }
                KeyCode::Enter => {
                    let selected_config = self.filtered.state.selected().unwrap_or_default();
                    debug!(
                        "Selected config: {}",
                        self.filtered.items[selected_config].name
                    );

                    return (
                        Some(event.clone()),
                        vec![WorkerMessage::ConfigSelected(
                            self.filtered.state.selected().unwrap_or_default(),
                        )],
                    );
                }
                _ => return (Some(FlowrsEvent::Key(*key_event)), vec![]),
            }
        }
        (None, vec![])
    }

    fn view(&mut self, f: &mut Frame) {
        let rects = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .margin(0)
            .split(f.area());

        let selected_style = DEFAULT_STYLE.add_modifier(Modifier::REVERSED);

        let headers = ["Name", "Endpoint", "Managed"];
        let header_cells = headers.iter().map(|h| Cell::from(*h).style(DEFAULT_STYLE));

        let header =
            Row::new(header_cells).style(DEFAULT_STYLE.reversed().add_modifier(Modifier::BOLD));

        let rows = self.filtered.items.iter().enumerate().map(|(idx, item)| {
            Row::new(vec![
                Line::from(item.name.as_str()),
                Line::from(item.endpoint.as_str()),
                Line::from(if let Some(managed_service) = &item.managed {
                    managed_service.to_string()
                } else {
                    "None".to_string()
                }),
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
                Constraint::Percentage(20),
                Constraint::Percentage(80),
                Constraint::Percentage(20),
            ],
        )
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Config"))
        .style(DEFAULT_STYLE)
        .row_highlight_style(selected_style);
        f.render_stateful_widget(t, rects[0], &mut self.filtered.state);
    }
}
