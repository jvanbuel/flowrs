use crossterm::event::KeyCode;
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Row, StatefulWidget, Table, Widget};

use crate::airflow::config::AirflowConfig;
use crate::app::events::custom::FlowrsEvent;
use crate::app::worker::WorkerMessage;
use crate::ui::constants::{ALTERNATING_ROW_COLOR, DEFAULT_STYLE};

use super::{filter::Filter, Model, StatefulTable};
use crate::app::error::FlowrsError;
use crate::ui::common::create_headers;

pub struct ConfigModel {
    pub all: Vec<AirflowConfig>,
    pub filtered: StatefulTable<AirflowConfig>,
    pub filter: Filter,
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
        match event {
            FlowrsEvent::Tick => return (Some(FlowrsEvent::Tick), vec![]),
            FlowrsEvent::Key(key_event) => {
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
                    _ => (),
                }
                return (Some(event.clone()), vec![]);
            }
            _ => (),
        };
        (Some(event.clone()), vec![])
    }
}

impl Widget for &mut ConfigModel {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let selected_style = DEFAULT_STYLE.add_modifier(Modifier::REVERSED);

        let headers = ["Name", "Endpoint", "Managed"];
        let header_row = create_headers(headers);

        let header =
            Row::new(header_row).style(DEFAULT_STYLE.reversed().add_modifier(Modifier::BOLD));

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
                DEFAULT_STYLE.bg(ALTERNATING_ROW_COLOR)
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
        StatefulWidget::render(t, area, buf, &mut self.filtered.state);
    }
}
