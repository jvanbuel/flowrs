use std::ops::RangeInclusive;
use std::vec;

use super::popup::commands_help::CommandPopUp;
use super::popup::error::ErrorPopup;
use super::popup::taskinstances::commands::TASK_COMMAND_POP_UP;
use crossterm::event::KeyCode;
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, Row, StatefulWidget, Table, Widget};
use time::format_description;

use crate::airflow::model::common::TaskInstance;
use crate::app::events::custom::FlowrsEvent;
use crate::ui::common::{create_headers, state_to_colored_square};
use crate::ui::constants::AirflowStateColor;
use crate::ui::theme::{
    ALT_ROW_STYLE, BORDER_STYLE, DEFAULT_STYLE, MARKED_STYLE, SELECTED_ROW_STYLE,
    TABLE_HEADER_STYLE, TITLE_STYLE,
};
use crate::ui::TIME_FORMAT;

use super::popup::taskinstances::clear::ClearTaskInstancePopup;
use super::popup::taskinstances::mark::MarkTaskInstancePopup;
use super::popup::taskinstances::TaskInstancePopUp;
use super::{filter::Filter, Model, StatefulTable};
use crate::app::worker::{OpenItem, WorkerMessage};

pub struct TaskInstanceModel {
    pub dag_id: Option<String>,
    pub dag_run_id: Option<String>,
    pub all: Vec<TaskInstance>,
    pub filtered: StatefulTable<TaskInstance>,
    pub filter: Filter,
    pub popup: Option<TaskInstancePopUp>,
    pub visual_mode: bool,
    pub visual_anchor: Option<usize>,
    commands: Option<&'static CommandPopUp<'static>>,
    pub error_popup: Option<ErrorPopup>,
    ticks: u32,
    event_buffer: Vec<FlowrsEvent>,
}

impl TaskInstanceModel {
    pub fn new() -> Self {
        TaskInstanceModel {
            dag_id: None,
            dag_run_id: None,
            all: vec![],
            filtered: StatefulTable::new(vec![]),
            filter: Filter::new(),
            popup: None,
            visual_mode: false,
            visual_anchor: None,
            commands: None,
            error_popup: None,
            ticks: 0,
            event_buffer: vec![],
        }
    }

    pub fn filter_task_instances(&mut self) {
        let prefix = &self.filter.prefix;
        let filtered_task_instances = match prefix {
            Some(prefix) => &self
                .all
                .iter()
                .filter(|task_instance| task_instance.task_id.contains(prefix))
                .cloned()
                .collect::<Vec<TaskInstance>>(),
            None => &self.all,
        };
        self.filtered.items = filtered_task_instances.clone();
    }

    #[allow(dead_code)]
    pub fn current(&mut self) -> Option<&mut TaskInstance> {
        self.filtered
            .state
            .selected()
            .map(|i| &mut self.filtered.items[i])
    }
    pub fn mark_task_instance(&mut self, task_id: &str, status: &str) {
        self.filtered.items.iter_mut().for_each(|task_instance| {
            if task_instance.task_id == task_id {
                task_instance.state = Some(status.to_string());
            }
        });
    }

    /// Returns the inclusive range of selected indices, if in visual mode
    fn visual_selection(&self) -> Option<RangeInclusive<usize>> {
        if !self.visual_mode {
            return None;
        }
        let anchor = self.visual_anchor?;
        let cursor = self.filtered.state.selected()?;
        let (start, end) = if anchor <= cursor {
            (anchor, cursor)
        } else {
            (cursor, anchor)
        };
        Some(start..=end)
    }

    /// Returns count of selected items (for bottom border display)
    fn visual_selection_count(&self) -> usize {
        self.visual_selection()
            .map_or(0, |r| r.end() - r.start() + 1)
    }

    /// Returns selected task IDs for passing to mark popup
    fn selected_task_ids(&self) -> Vec<String> {
        match self.visual_selection() {
            Some(range) => range
                .filter_map(|i| self.filtered.items.get(i))
                .map(|item| item.task_id.clone())
                .collect(),
            None => {
                // Normal mode: just current item
                self.filtered
                    .state
                    .selected()
                    .and_then(|i| self.filtered.items.get(i))
                    .map(|item| vec![item.task_id.clone()])
                    .unwrap_or_default()
            }
        }
    }
}

impl Default for TaskInstanceModel {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for TaskInstanceModel {
    fn update(&mut self, event: &FlowrsEvent) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        match event {
            FlowrsEvent::Tick => {
                self.ticks += 1;
                if !self.ticks.is_multiple_of(10) {
                    return (Some(FlowrsEvent::Tick), vec![]);
                }
                if let (Some(dag_run_id), Some(dag_id)) = (&self.dag_run_id, &self.dag_id) {
                    log::debug!("Updating task instances for dag_run_id: {dag_run_id}");
                    return (
                        Some(FlowrsEvent::Tick),
                        vec![WorkerMessage::UpdateTaskInstances {
                            dag_id: dag_id.clone(),
                            dag_run_id: dag_run_id.clone(),
                            clear: false,
                        }],
                    );
                }
                (Some(FlowrsEvent::Tick), vec![])
            }
            FlowrsEvent::Key(key_event) => {
                if self.filter.is_enabled() {
                    self.filter.update(key_event);
                    self.filter_task_instances();
                    return (None, vec![]);
                } else if let Some(_error_popup) = &mut self.error_popup {
                    match key_event.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            self.error_popup = None;
                        }
                        _ => (),
                    }
                    return (None, vec![]);
                } else if let Some(_commands) = &mut self.commands {
                    match key_event.code {
                        KeyCode::Char('q' | '?') | KeyCode::Esc => {
                            self.commands = None;
                        }
                        _ => (),
                    }
                } else if let Some(popup) = &mut self.popup {
                    match popup {
                        TaskInstancePopUp::Clear(popup) => {
                            let (key_event, messages) = popup.update(event);
                            debug!("Popup messages: {messages:?}");
                            if let Some(FlowrsEvent::Key(key_event)) = &key_event {
                                match key_event.code {
                                    KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => {
                                        self.popup = None;
                                        self.visual_mode = false;
                                        self.visual_anchor = None;
                                    }
                                    _ => {}
                                }
                            }
                            return (None, messages);
                        }
                        TaskInstancePopUp::Mark(popup) => {
                            let (key_event, messages) = popup.update(event);
                            debug!("Popup messages: {messages:?}");
                            if let Some(FlowrsEvent::Key(key_event)) = &key_event {
                                match key_event.code {
                                    KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => {
                                        self.popup = None;
                                        self.visual_mode = false;
                                        self.visual_anchor = None;
                                    }
                                    _ => {}
                                }
                            }
                            return (None, messages);
                        }
                    }
                } else {
                    match key_event.code {
                        KeyCode::Down | KeyCode::Char('j') => {
                            self.filtered.next();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.filtered.previous();
                        }
                        KeyCode::Char('G') => {
                            if !self.filtered.items.is_empty() {
                                self.filtered
                                    .state
                                    .select(Some(self.filtered.items.len() - 1));
                            }
                        }
                        KeyCode::Char('g') => {
                            if let Some(FlowrsEvent::Key(key_event)) = self.event_buffer.pop() {
                                if key_event.code == KeyCode::Char('g') {
                                    self.filtered.state.select_first();
                                } else {
                                    self.event_buffer.push(FlowrsEvent::Key(key_event));
                                }
                            } else {
                                self.event_buffer.push(FlowrsEvent::Key(*key_event));
                            }
                        }
                        KeyCode::Char('V') => {
                            if let Some(cursor) = self.filtered.state.selected() {
                                self.visual_mode = true;
                                self.visual_anchor = Some(cursor);
                            }
                        }
                        KeyCode::Esc => {
                            if self.visual_mode {
                                self.visual_mode = false;
                                self.visual_anchor = None;
                                return (None, vec![]);
                            }
                            return (Some(FlowrsEvent::Key(*key_event)), vec![]);
                        }
                        KeyCode::Char('m') => {
                            let task_ids = self.selected_task_ids();
                            if !task_ids.is_empty() {
                                if let (Some(dag_id), Some(dag_run_id)) =
                                    (&self.dag_id, &self.dag_run_id)
                                {
                                    self.popup =
                                        Some(TaskInstancePopUp::Mark(MarkTaskInstancePopup::new(
                                            task_ids,
                                            dag_id,
                                            dag_run_id,
                                        )));
                                }
                            }
                        }
                        KeyCode::Char('c') => {
                            let task_ids = self.selected_task_ids();
                            if let (Some(dag_id), Some(dag_run_id)) =
                                (&self.dag_id, &self.dag_run_id)
                            {
                                if !task_ids.is_empty() {
                                    self.popup =
                                        Some(TaskInstancePopUp::Clear(ClearTaskInstancePopup::new(
                                            dag_run_id,
                                            dag_id,
                                            task_ids,
                                        )));
                                }
                            }
                        }
                        KeyCode::Char('?') => {
                            self.commands = Some(&*TASK_COMMAND_POP_UP);
                        }
                        KeyCode::Char('/') => {
                            self.filter.toggle();
                            self.filter_task_instances();
                        }
                        KeyCode::Enter => {
                            if let Some(task_instance) = self.current() {
                                return (
                                    Some(FlowrsEvent::Key(*key_event)),
                                    vec![WorkerMessage::UpdateTaskLogs {
                                        dag_id: task_instance.dag_id.clone(),
                                        dag_run_id: task_instance.dag_run_id.clone(),
                                        task_id: task_instance.task_id.clone(),
                                        #[allow(
                                            clippy::cast_sign_loss,
                                            clippy::cast_possible_truncation
                                        )]
                                        task_try: task_instance.try_number as u16,
                                        clear: true,
                                    }],
                                );
                            }
                        }
                        KeyCode::Char('o') => {
                            if let Some(task_instance) = self.current() {
                                return (
                                    Some(FlowrsEvent::Key(*key_event)),
                                    vec![WorkerMessage::OpenItem(OpenItem::TaskInstance {
                                        dag_id: task_instance.dag_id.clone(),
                                        dag_run_id: task_instance.dag_run_id.clone(),
                                        task_id: task_instance.task_id.clone(),
                                    })],
                                );
                            }
                        }
                        _ => return (Some(FlowrsEvent::Key(*key_event)), vec![]), // if no match, return the event
                    }
                }
                (None, vec![])
            }
            FlowrsEvent::Mouse => (Some(event.clone()), vec![]),
        }
    }
}
impl Widget for &mut TaskInstanceModel {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let rects = if self.filter.is_enabled() {
            let rects = Layout::default()
                .constraints([Constraint::Fill(90), Constraint::Max(3)].as_ref())
                .margin(0)
                .split(area);

            self.filter.render(rects[1], buffer);
            rects
        } else {
            Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .margin(0)
                .split(area)
        };

        let headers = ["Task ID", "Execution Date", "Duration", "State", "Tries"];
        let header_row = create_headers(headers);
        let header = Row::new(header_row).style(TABLE_HEADER_STYLE);

        let rows = self.filtered.items.iter().enumerate().map(|(idx, item)| {
            Row::new(vec![
                Line::from(item.task_id.as_str()),
                Line::from(if let Some(date) = item.logical_date {
                    date.format(&format_description::parse(TIME_FORMAT).unwrap())
                        .unwrap()
                        .clone()
                } else {
                    "None".to_string()
                }),
                Line::from(if let Some(i) = item.duration {
                    format!("{i}")
                } else {
                    "None".to_string()
                }),
                Line::from(if let Some(state) = &item.state {
                    match state.as_str() {
                        "success" => state_to_colored_square(AirflowStateColor::Success),
                        "running" => state_to_colored_square(AirflowStateColor::Running),
                        "failed" => state_to_colored_square(AirflowStateColor::Failed),
                        "queued" => state_to_colored_square(AirflowStateColor::Queued),
                        "up_for_retry" => state_to_colored_square(AirflowStateColor::UpForRetry),
                        "upstream_failed" => {
                            state_to_colored_square(AirflowStateColor::UpstreamFailed)
                        }
                        _ => state_to_colored_square(AirflowStateColor::None),
                    }
                } else {
                    state_to_colored_square(AirflowStateColor::None)
                }),
                Line::from(format!("{:?}", item.try_number)),
            ])
            .style(
                if self
                    .visual_selection()
                    .is_some_and(|r| r.contains(&idx))
                {
                    MARKED_STYLE
                } else if (idx % 2) == 0 {
                    DEFAULT_STYLE
                } else {
                    ALT_ROW_STYLE
                },
            )
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
                .border_type(BorderType::Plain)
                .borders(Borders::ALL)
                .title(" TaskInstances - Press <?> to see available commands ")
                .border_style(BORDER_STYLE)
                .title_style(TITLE_STYLE);
            if self.visual_mode {
                block.title_bottom(format!(
                    " -- VISUAL ({} selected) -- ",
                    self.visual_selection_count()
                ))
            } else {
                block
            }
        })
        .row_highlight_style(SELECTED_ROW_STYLE);

        StatefulWidget::render(t, rects[0], buffer, &mut self.filtered.state);

        match &mut self.popup {
            Some(TaskInstancePopUp::Clear(popup)) => {
                popup.render(area, buffer);
            }
            Some(TaskInstancePopUp::Mark(popup)) => {
                popup.render(area, buffer);
            }
            _ => (),
        }

        if let Some(commands) = &self.commands {
            commands.render(area, buffer);
        }

        if let Some(error_popup) = &self.error_popup {
            error_popup.render(area, buffer);
        }
    }
}
