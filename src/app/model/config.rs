use crossterm::event::{KeyCode, KeyEvent};
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Row, StatefulWidget, Table, Widget};

use crate::airflow::config::AirflowConfig;
use crate::app::events::custom::FlowrsEvent;
use crate::app::worker::{OpenItem, WorkerMessage};
use crate::ui::theme::{
    ACCENT, ALT_ROW_STYLE, BORDER_STYLE, DEFAULT_STYLE, SELECTED_ROW_STYLE, TABLE_HEADER_STYLE,
};

use super::popup::commands_help::CommandPopUp;
use super::popup::config::commands::CONFIG_COMMAND_POP_UP;
use super::popup::error::ErrorPopup;
use super::{dismiss_commands_popup, dismiss_error_popup, FilterableTable, KeyResult, Model};
use crate::ui::common::create_headers;

pub struct ConfigModel {
    /// Filterable table containing all configs and filtered view
    pub table: FilterableTable<AirflowConfig>,
    pub commands: Option<&'static CommandPopUp<'static>>,
    pub error_popup: Option<ErrorPopup>,
    event_buffer: Vec<KeyCode>,
}

impl ConfigModel {
    pub fn new(configs: &[AirflowConfig]) -> Self {
        let mut table = FilterableTable::new();
        table.set_items(configs.to_vec());
        let config_names: Vec<String> = configs.iter().map(|c| c.name.clone()).collect();
        table.set_primary_values("name", config_names);

        Self {
            table,
            commands: None,
            error_popup: None,
            event_buffer: Vec::new(),
        }
    }

    pub fn new_with_errors(configs: &[AirflowConfig], errors: Vec<String>) -> Self {
        let error_popup = if errors.is_empty() {
            None
        } else {
            Some(ErrorPopup::from_strings(errors))
        };

        let mut table = FilterableTable::new();
        table.set_items(configs.to_vec());
        let config_names: Vec<String> = configs.iter().map(|c| c.name.clone()).collect();
        table.set_primary_values("name", config_names);

        Self {
            table,
            commands: None,
            error_popup,
            event_buffer: Vec::new(),
        }
    }

    /// Apply filter to configs
    pub fn filter_configs(&mut self) {
        self.table.apply_filter();
    }

    /// Handle model-specific keys
    fn handle_keys(&mut self, key_event: &KeyEvent) -> KeyResult {
        match key_event.code {
            KeyCode::Char('o') => {
                let selected_config = self.table.filtered.state.selected().unwrap_or_default();
                let endpoint = self.table.filtered.items[selected_config].endpoint.clone();
                KeyResult::PassWith(vec![WorkerMessage::OpenItem(OpenItem::Config(endpoint))])
            }
            KeyCode::Char('?') => {
                self.commands = Some(&*CONFIG_COMMAND_POP_UP);
                KeyResult::Consumed
            }
            KeyCode::Enter => {
                let selected_config = self.table.filtered.state.selected().unwrap_or_default();
                debug!(
                    "Selected config: {}",
                    self.table.filtered.items[selected_config].name
                );
                KeyResult::PassWith(vec![WorkerMessage::ConfigSelected(selected_config)])
            }
            _ => KeyResult::PassThrough,
        }
    }
}

impl Model for ConfigModel {
    fn update(&mut self, event: &FlowrsEvent) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        match event {
            FlowrsEvent::Tick => (Some(FlowrsEvent::Tick), vec![]),
            FlowrsEvent::Key(key_event) => {
                let result = self
                    .table
                    .handle_filter_key(key_event)
                    .or_else(|| dismiss_error_popup(&mut self.error_popup, key_event.code))
                    .or_else(|| dismiss_commands_popup(&mut self.commands, key_event.code))
                    .or_else(|| self.table.handle_navigation(key_event.code, &mut self.event_buffer))
                    .or_else(|| self.handle_keys(key_event));

                result.into_result(event)
            }
            FlowrsEvent::Mouse | FlowrsEvent::FocusGained | FlowrsEvent::FocusLost => {
                (Some(event.clone()), vec![])
            }
        }
    }
}

impl Widget for &mut ConfigModel {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rects = if self.table.is_filter_active() {
            let rects = Layout::default()
                .constraints([Constraint::Fill(90), Constraint::Max(3)].as_ref())
                .margin(0)
                .split(area);

            self.table.filter.render_widget(rects[1], buf);
            rects
        } else {
            Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .margin(0)
                .split(area)
        };
        let headers = ["Name", "Endpoint", "Managed", "Version"];
        let header_row = create_headers(headers);
        let header = Row::new(header_row).style(TABLE_HEADER_STYLE);

        let rows = self
            .table
            .filtered
            .items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                Row::new(vec![
                    Line::from(item.name.as_str()),
                    Line::from(item.endpoint.as_str()),
                    Line::from(
                        item.managed
                            .as_ref()
                            .map_or_else(|| "None".to_string(), ToString::to_string),
                    ),
                    Line::from(match item.version {
                        crate::airflow::config::AirflowVersion::V2 => "v2",
                        crate::airflow::config::AirflowVersion::V3 => "v3",
                    }),
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
                Constraint::Percentage(20),
                Constraint::Percentage(55),
                Constraint::Percentage(15),
                Constraint::Min(8),
            ],
        )
        .header(header)
        .block({
            let block = Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                .border_style(BORDER_STYLE)
                .title(" Press <?> to see available commands ");
            if let Some(filter_text) = self.table.filter.filter_display() {
                block.title_bottom(Line::from(Span::styled(
                    format!(" Filter: {filter_text} "),
                    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                )))
            } else {
                block
            }
        })
        .row_highlight_style(SELECTED_ROW_STYLE);
        StatefulWidget::render(t, rects[0], buf, &mut self.table.filtered.state);

        if let Some(commands) = &self.commands {
            commands.render(area, buf);
        }

        if let Some(error_popup) = &self.error_popup {
            error_popup.render(area, buf);
        }
    }
}
