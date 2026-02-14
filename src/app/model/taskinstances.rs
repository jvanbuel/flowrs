use std::vec;

use super::popup::taskinstances::commands::TASK_COMMAND_POP_UP;
use crossterm::event::KeyCode;
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, Row, StatefulWidget, Table, Widget};
use time::format_description;

use crate::airflow::graph::{sort_task_instances, TaskGraph};
use crate::airflow::model::common::{calculate_duration, format_duration, TaskId, TaskInstance};
use crate::app::events::custom::FlowrsEvent;
use crate::ui::common::{create_headers, state_to_colored_square};
use crate::ui::constants::AirflowStateColor;
use crate::ui::theme::{BORDER_STYLE, SELECTED_ROW_STYLE, TABLE_HEADER_STYLE};
use crate::ui::TIME_FORMAT;

use super::popup::taskinstances::clear::ClearTaskInstancePopup;
use super::popup::taskinstances::mark::MarkTaskInstancePopup;
use super::popup::taskinstances::TaskInstancePopUp;
use super::{FilterableTable, KeyResult, Model, Popup};
use crate::app::worker::{OpenItem, WorkerMessage};

/// Model for the Task Instance panel, managing the list of task instances and their filtering.
pub struct TaskInstanceModel {
    /// Filterable table containing all task instances and filtered view
    pub table: FilterableTable<TaskInstance>,
    /// Unified popup state (error, commands, or custom for this model)
    pub popup: Popup<TaskInstancePopUp>,
    ticks: u32,
    event_buffer: Vec<KeyCode>,
    pub task_graph: Option<TaskGraph>,
}

impl Default for TaskInstanceModel {
    fn default() -> Self {
        Self {
            table: FilterableTable::new(),
            popup: Popup::None,
            ticks: 0,
            event_buffer: Vec::new(),
            task_graph: None,
        }
    }
}

impl TaskInstanceModel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sort task instances by topological order (or timestamp fallback)
    pub fn sort_task_instances(&mut self) {
        if let Some(graph) = &self.task_graph {
            sort_task_instances(&mut self.table.all, graph);
        }
    }

    /// Mark a task instance with a new status (optimistic update)
    pub fn mark_task_instance(&mut self, task_id: &TaskId, status: &str) {
        if let Some(task_instance) = self
            .table
            .filtered
            .items
            .iter_mut()
            .find(|ti| ti.task_id == *task_id)
        {
            task_instance.state = Some(status.to_string());
        }
    }

    /// Returns selected task IDs for passing to mark/clear popups
    fn selected_task_ids(&self) -> Vec<TaskId> {
        self.table.selected_ids(|item| item.task_id.clone())
    }
}

impl TaskInstanceModel {
    /// Handle model-specific popups (returns messages from popup)
    fn handle_popup(
        &mut self,
        event: &FlowrsEvent,
        ctx: &crate::app::state::NavigationContext,
    ) -> Option<Vec<WorkerMessage>> {
        let custom_popup = self.popup.custom_mut()?;
        let (key_event, messages) = match custom_popup {
            TaskInstancePopUp::Clear(p) => p.update(event, ctx),
            TaskInstancePopUp::Mark(p) => p.update(event, ctx),
        };
        debug!("Popup messages: {messages:?}");

        if let Some(FlowrsEvent::Key(key_event)) = &key_event {
            if matches!(
                key_event.code,
                KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q')
            ) {
                self.popup.close();
                self.table.visual_mode = false;
                self.table.visual_anchor = None;
            }
        }
        Some(messages)
    }

    /// Handle model-specific keys
    fn handle_keys(
        &mut self,
        key_code: KeyCode,
        ctx: &crate::app::state::NavigationContext,
    ) -> KeyResult {
        match key_code {
            KeyCode::Char('m') => {
                let task_ids = self.selected_task_ids();
                if !task_ids.is_empty() {
                    if let (Some(dag_id), Some(dag_run_id)) = (&ctx.dag_id, &ctx.dag_run_id) {
                        self.popup.show_custom(TaskInstancePopUp::Mark(
                            MarkTaskInstancePopup::new(task_ids, dag_id, dag_run_id),
                        ));
                    }
                }
                KeyResult::Consumed
            }
            KeyCode::Char('c') => {
                let task_ids = self.selected_task_ids();
                if let (Some(dag_id), Some(dag_run_id)) = (&ctx.dag_id, &ctx.dag_run_id) {
                    if !task_ids.is_empty() {
                        self.popup.show_custom(TaskInstancePopUp::Clear(
                            ClearTaskInstancePopup::new(dag_run_id, dag_id, task_ids),
                        ));
                    }
                }
                KeyResult::Consumed
            }
            KeyCode::Char('?') => {
                self.popup.show_commands(&TASK_COMMAND_POP_UP);
                KeyResult::Consumed
            }
            KeyCode::Enter => {
                if let Some(task_instance) = self.table.current() {
                    KeyResult::PassWith(vec![WorkerMessage::UpdateTaskLogs {
                        dag_id: task_instance.dag_id.clone(),
                        dag_run_id: task_instance.dag_run_id.clone(),
                        task_id: task_instance.task_id.clone(),
                        task_try: task_instance.try_number,
                    }])
                } else {
                    KeyResult::Consumed
                }
            }
            KeyCode::Char('o') => {
                if let Some(task_instance) = self.table.current() {
                    KeyResult::PassWith(vec![WorkerMessage::OpenItem(OpenItem::TaskInstance {
                        dag_id: task_instance.dag_id.clone(),
                        dag_run_id: task_instance.dag_run_id.clone(),
                        task_id: task_instance.task_id.clone(),
                    })])
                } else {
                    KeyResult::Consumed
                }
            }
            _ => KeyResult::PassThrough,
        }
    }
}

impl Model for TaskInstanceModel {
    fn update(
        &mut self,
        event: &FlowrsEvent,
        ctx: &crate::app::state::NavigationContext,
    ) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        match event {
            FlowrsEvent::Tick => {
                self.ticks += 1;
                if !self.ticks.is_multiple_of(10) {
                    return (Some(FlowrsEvent::Tick), vec![]);
                }
                if let (Some(dag_id), Some(dag_run_id)) = (&ctx.dag_id, &ctx.dag_run_id) {
                    log::debug!("Updating task instances for dag_run_id: {dag_run_id}");
                    return (
                        Some(FlowrsEvent::Tick),
                        vec![WorkerMessage::UpdateTaskInstances {
                            dag_id: dag_id.clone(),
                            dag_run_id: dag_run_id.clone(),
                        }],
                    );
                }
                (Some(FlowrsEvent::Tick), vec![])
            }
            FlowrsEvent::Key(key_event) => {
                // Popup handling (has its own update method)
                if let Some(messages) = self.handle_popup(event, ctx) {
                    return (None, messages);
                }

                // Chain the handlers
                let result = self
                    .table
                    .handle_filter_key(key_event)
                    .or_else(|| self.popup.handle_dismiss(key_event.code))
                    .or_else(|| self.table.handle_visual_mode_key(key_event.code))
                    .or_else(|| {
                        self.table
                            .handle_navigation(key_event.code, &mut self.event_buffer)
                    })
                    .or_else(|| self.handle_keys(key_event.code, ctx));

                result.into_result(event)
            }
            FlowrsEvent::Mouse | FlowrsEvent::FocusGained | FlowrsEvent::FocusLost => {
                (Some(event.clone()), vec![])
            }
        }
    }
}
impl Widget for &mut TaskInstanceModel {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let content_area = self.table.render_with_filter(area, buffer);

        let headers = ["Task ID", "Execution Date", "Duration", "State", "Tries"];
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
                    Line::from(item.task_id.as_ref()),
                    Line::from(if let Some(date) = item.logical_date {
                        date.format(
                            &format_description::parse(TIME_FORMAT)
                                .expect("TIME_FORMAT constant should be a valid time format"),
                        )
                        .expect("Date formatting with TIME_FORMAT should succeed")
                    } else {
                        "None".to_string()
                    }),
                    Line::from(
                        calculate_duration(item).map_or_else(|| "-".to_string(), format_duration),
                    ),
                    Line::from(if let Some(state) = &item.state {
                        match state.as_str() {
                            "success" => state_to_colored_square(AirflowStateColor::Success),
                            "running" => state_to_colored_square(AirflowStateColor::Running),
                            "failed" => state_to_colored_square(AirflowStateColor::Failed),
                            "queued" => state_to_colored_square(AirflowStateColor::Queued),
                            "up_for_retry" => {
                                state_to_colored_square(AirflowStateColor::UpForRetry)
                            }
                            "upstream_failed" => {
                                state_to_colored_square(AirflowStateColor::UpstreamFailed)
                            }
                            _ => state_to_colored_square(AirflowStateColor::None),
                        }
                    } else {
                        state_to_colored_square(AirflowStateColor::None)
                    }),
                    Line::from(item.try_number.to_string()),
                ])
                .style(self.table.row_style(idx))
            });
        let t = Table::new(
            rows,
            &[
                Constraint::Fill(1),
                Constraint::Min(19),
                Constraint::Length(20),
                Constraint::Length(5),
                Constraint::Length(5),
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

        StatefulWidget::render(t, content_area, buffer, &mut self.table.filtered.state);

        // Render any active popup (error, commands, or custom)
        (&self.popup).render(area, buffer);

        // Render custom popups that need special handling
        match self.popup.custom_mut() {
            Some(TaskInstancePopUp::Clear(popup)) => popup.render(area, buffer),
            Some(TaskInstancePopUp::Mark(popup)) => popup.render(area, buffer),
            None => {}
        }
    }
}
