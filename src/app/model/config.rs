use crossterm::event::{KeyCode, KeyEvent};
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, Row, StatefulWidget, Table, Widget};

use crate::airflow::config::AirflowConfig;
use crate::app::events::custom::FlowrsEvent;
use crate::app::worker::{OpenItem, WorkerMessage};
use crate::ui::theme::{BORDER_STYLE, SELECTED_ROW_STYLE, TABLE_HEADER_STYLE};

use super::popup::config::commands::CONFIG_COMMAND_POP_UP;
use super::{FilterableTable, KeyResult, Model, Popup};
use crate::ui::common::create_headers;

pub struct ConfigModel {
    /// Filterable table containing all configs and filtered view
    pub table: FilterableTable<AirflowConfig>,
    /// Unified popup state (error, commands, or none for this model)
    pub popup: Popup,
    event_buffer: Vec<KeyCode>,
}

impl ConfigModel {
    pub fn new(configs: &[AirflowConfig]) -> Self {
        let mut table = FilterableTable::new();
        table.set_items(configs.to_vec());
        let config_names: Vec<String> = configs.iter().map(|c| c.name.clone()).collect();
        table.filter.set_primary_values("name", config_names);

        Self {
            table,
            popup: Popup::None,
            event_buffer: Vec::new(),
        }
    }

    pub fn new_with_errors(configs: &[AirflowConfig], errors: Vec<String>) -> Self {
        let mut model = Self::new(configs);
        if !errors.is_empty() {
            model.popup.show_error(errors);
        }
        model
    }

    /// Handle model-specific keys
    fn handle_keys(&mut self, key_event: &KeyEvent) -> KeyResult {
        match key_event.code {
            KeyCode::Char('o') => {
                if let Some(idx) = self.table.filtered.state.selected() {
                    if let Some(item) = self.table.filtered.items.get(idx) {
                        return KeyResult::PassWith(vec![WorkerMessage::OpenItem(
                            OpenItem::Config(item.endpoint.clone()),
                        )]);
                    }
                }
                KeyResult::PassThrough
            }
            KeyCode::Char('?') => {
                self.popup.show_commands(&CONFIG_COMMAND_POP_UP);
                KeyResult::Consumed
            }
            KeyCode::Enter => {
                if let Some(idx) = self.table.filtered.state.selected() {
                    if let Some(item) = self.table.filtered.items.get(idx) {
                        debug!("Selected config: {}", item.name);
                        return KeyResult::PassWith(vec![WorkerMessage::ConfigSelected(idx)]);
                    }
                }
                KeyResult::PassThrough
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
                    .or_else(|| self.popup.handle_dismiss(key_event.code))
                    .or_else(|| {
                        self.table
                            .handle_navigation(key_event.code, &mut self.event_buffer)
                    })
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
        let content_area = self.table.render_with_filter(area, buf);

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
                .style(self.table.row_style(idx))
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
            if let Some(title) = self.table.status_title() {
                block.title_bottom(title)
            } else {
                block
            }
        })
        .row_highlight_style(SELECTED_ROW_STYLE);
        StatefulWidget::render(t, content_area, buf, &mut self.table.filtered.state);

        // Render any active popup (error or commands)
        (&self.popup).render(area, buf);
    }
}
