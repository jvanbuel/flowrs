pub mod commands;
pub mod popup;
mod render;

use std::collections::HashSet;

use commands::TASK_COMMAND_POP_UP;
use crossterm::event::KeyCode;
use log::debug;

use crate::airflow::graph::{sort_task_instances, TaskGraph};
use crate::airflow::model::common::{
    DagId, DagRunId, GanttData, TaskId, TaskInstance, TaskInstanceState,
};
use crate::app::events::custom::FlowrsEvent;

use super::{FilterableTable, KeyResult, Model, Popup};
use crate::airflow::model::common::OpenItem;
use crate::app::worker::WorkerMessage;
use popup::clear::ClearTaskInstancePopup;
use popup::mark::MarkTaskInstancePopup;
use popup::TaskInstancePopUp;

/// Model for the Task Instance panel, managing the list of task instances and their filtering.
pub struct TaskInstanceModel {
    /// Filterable table containing all task instances and filtered view
    pub table: FilterableTable<TaskInstance>,
    /// Unified popup state (error, commands, or custom for this model)
    pub popup: Popup<TaskInstancePopUp>,
    /// Gantt chart data computed from task instances and their tries
    pub gantt_data: GanttData,
    /// Tracks which DAG + run the cached `gantt_data` belongs to, so we can
    /// invalidate it when the user navigates to a different DAG or run.
    current_gantt_key: Option<(DagId, DagRunId)>,
    ticks: u32,
    poll_tick_multiplier: u32,
    event_buffer: Vec<KeyCode>,
    pub task_graph: Option<TaskGraph>,
}

impl Default for TaskInstanceModel {
    fn default() -> Self {
        Self {
            table: FilterableTable::new(),
            popup: Popup::None,
            gantt_data: GanttData::default(),
            current_gantt_key: None,
            ticks: 0,
            poll_tick_multiplier: 10,
            event_buffer: Vec::new(),
            task_graph: None,
        }
    }
}

impl TaskInstanceModel {
    pub fn new(poll_tick_multiplier: u32) -> Self {
        Self {
            poll_tick_multiplier,
            ..Self::default()
        }
    }

    /// Notify the model which DAG + run is now active.
    pub fn set_gantt_context(&mut self, dag_id: &DagId, dag_run_id: &DagRunId) {
        let key = (dag_id.clone(), dag_run_id.clone());
        if self.current_gantt_key.as_ref() != Some(&key) {
            self.gantt_data = GanttData::default();
            self.current_gantt_key = Some(key);
        }
    }

    /// Sort task instances by topological order (or timestamp fallback)
    pub fn sort_task_instances(&mut self) {
        if let Some(graph) = &self.task_graph {
            sort_task_instances(&mut self.table.all, graph);
        }
    }

    /// Rebuild Gantt data from the current task instance list.
    /// Returns task IDs that have retries (`try_number` > 1) for fetching detailed tries.
    pub fn rebuild_gantt(&mut self) -> Vec<TaskId> {
        let (new_gantt, retried) = Self::build_gantt(&self.table.all, &self.gantt_data);
        self.gantt_data = new_gantt;
        retried
    }

    /// Build Gantt data from task instances without requiring `&mut self`.
    /// This allows building the gantt outside a lock and storing it later.
    /// Returns the new gantt data and task IDs that have retries.
    pub fn build_gantt(
        task_instances: &[TaskInstance],
        existing_gantt: &GanttData,
    ) -> (GanttData, Vec<TaskId>) {
        let mut new_gantt = GanttData::from_task_instances(task_instances);

        let mut seen = HashSet::new();
        let retried: Vec<TaskId> = task_instances
            .iter()
            .filter(|ti| ti.try_number > 1 && seen.insert(ti.task_id.clone()))
            .map(|ti| ti.task_id.clone())
            .collect();

        for task_id in &retried {
            if let Some(cached_tries) = existing_gantt.task_tries.get(task_id) {
                let new_tries = new_gantt.task_tries.get(task_id);
                if cached_tries.len() > new_tries.map_or(0, Vec::len) {
                    new_gantt
                        .task_tries
                        .insert(task_id.clone(), cached_tries.clone());
                }
            }
        }

        new_gantt.recompute_window();
        (new_gantt, retried)
    }

    /// Mark a task instance with a new status (optimistic update)
    pub fn mark_task_instance(&mut self, task_id: &TaskId, status: TaskInstanceState) {
        if let Some(task_instance) = self
            .table
            .filtered
            .items
            .iter_mut()
            .find(|ti| ti.task_id == *task_id)
        {
            task_instance.state = Some(status);
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
                    if let (Some(dag_id), Some(dag_run_id)) = (ctx.dag_id(), ctx.dag_run_id()) {
                        self.popup.show_custom(TaskInstancePopUp::Mark(
                            MarkTaskInstancePopup::new(task_ids, dag_id, dag_run_id),
                        ));
                    }
                }
                KeyResult::Consumed
            }
            KeyCode::Char('c') => {
                let task_ids = self.selected_task_ids();
                if let (Some(dag_id), Some(dag_run_id)) = (ctx.dag_id(), ctx.dag_run_id()) {
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
                if !self.ticks.is_multiple_of(self.poll_tick_multiplier) {
                    return (Some(FlowrsEvent::Tick), vec![]);
                }
                if let (Some(dag_id), Some(dag_run_id)) = (ctx.dag_id(), ctx.dag_run_id()) {
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
                if let Some(messages) = self.handle_popup(event, ctx) {
                    return (None, messages);
                }

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
