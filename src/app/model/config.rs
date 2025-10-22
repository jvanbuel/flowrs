use crossterm::event::KeyCode;
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, Row, StatefulWidget, Table, Widget};

use crate::airflow::config::AirflowConfig;
use crate::app::events::custom::FlowrsEvent;
use crate::app::worker::{OpenItem, WorkerMessage};
use crate::ui::constants::{ALTERNATING_ROW_COLOR, DEFAULT_STYLE};

use super::popup::commands_help::CommandPopUp;
use super::popup::config::commands::CONFIG_COMMAND_POP_UP;
use super::popup::error::ErrorPopup;
use super::{filter::Filter, Model, StatefulTable};
use crate::ui::common::create_headers;

pub struct ConfigModel {
    pub all: Vec<AirflowConfig>,
    pub filtered: StatefulTable<AirflowConfig>,
    pub filter: Filter,
    pub commands: Option<&'static CommandPopUp<'static>>,
    pub error_popup: Option<ErrorPopup>,
}

impl ConfigModel {
    pub fn new(configs: Vec<AirflowConfig>) -> Self {
        ConfigModel {
            all: configs.clone(),
            filtered: StatefulTable::new(configs),
            filter: Filter::new(),
            commands: None,
            error_popup: None,
        }
    }

    pub fn new_with_errors(configs: Vec<AirflowConfig>, errors: Vec<String>) -> Self {
        let error_popup = if errors.is_empty() {
            None
        } else {
            Some(ErrorPopup::from_strings(errors))
        };

        ConfigModel {
            all: configs.clone(),
            filtered: StatefulTable::new(configs),
            filter: Filter::new(),
            commands: None,
            error_popup,
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
            None => dags.clone(),
        };
        self.filtered.items = filtered_configs;
    }
}

impl Model for ConfigModel {
    fn update(&mut self, event: &FlowrsEvent) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        match event {
            FlowrsEvent::Tick => (Some(FlowrsEvent::Tick), vec![]),
            FlowrsEvent::Key(key_event) => {
                if self.filter.enabled {
                    self.filter.update(key_event);
                    self.filter_configs();
                    return (None, vec![]);
                } else if self.error_popup.is_some() {
                    match key_event.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            self.error_popup = None;
                        }
                        _ => (),
                    }
                } else if let Some(_commands) = &mut self.commands {
                    match key_event.code {
                        KeyCode::Char('q' | '?') | KeyCode::Esc | KeyCode::Enter => {
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
                        KeyCode::Char('/') => {
                            self.filter.toggle();
                        }
                        KeyCode::Char('o') => {
                            let selected_config =
                                self.filtered.state.selected().unwrap_or_default();
                            let endpoint = self.filtered.items[selected_config].endpoint.clone();
                            return (
                                Some(event.clone()),
                                vec![WorkerMessage::OpenItem(OpenItem::Config(endpoint))],
                            );
                        }
                        KeyCode::Char('?') => {
                            self.commands = Some(&*CONFIG_COMMAND_POP_UP);
                        }
                        KeyCode::Enter => {
                            let selected_config =
                                self.filtered.state.selected().unwrap_or_default();
                            debug!(
                                "Selected config: {}",
                                self.filtered.items[selected_config].name
                            );

                            return (
                                Some(event.clone()),
                                vec![WorkerMessage::ConfigSelected(selected_config)],
                            );
                        }
                        _ => (),
                    }
                    return (Some(event.clone()), vec![]);
                }
                (None, vec![])
            }
            FlowrsEvent::Mouse => (Some(event.clone()), vec![]),
        }
    }
}

impl Widget for &mut ConfigModel {
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

        let headers = ["Name", "Endpoint", "Managed", "Version"];
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
                Line::from(match item.version {
                    crate::airflow::config::AirflowVersion::V2 => "v2",
                    crate::airflow::config::AirflowVersion::V3 => "v3",
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
                Constraint::Percentage(55),
                Constraint::Percentage(15),
                Constraint::Min(8),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .title("Config"),
        )
        .style(DEFAULT_STYLE)
        .row_highlight_style(selected_style);
        StatefulWidget::render(t, rects[0], buf, &mut self.filtered.state);

        if let Some(commands) = &self.commands {
            commands.render(area, buf);
        }

        if let Some(error_popup) = &self.error_popup {
            error_popup.render(area, buf);
        }
    }
}
