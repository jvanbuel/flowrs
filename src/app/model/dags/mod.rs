pub mod commands;
pub mod popup;
mod render;

use std::collections::HashMap;

use crossterm::event::KeyCode;
use log::debug;

use crate::airflow::model::common::{Dag, DagId, DagStatistic};
use crate::app::events::custom::FlowrsEvent;
use crate::app::model::dagruns::popup::trigger::TriggerDagRunPopUp;
use commands::DAG_COMMAND_POP_UP;

use super::dagruns::DagCodeView;
use super::{FilterableTable, KeyResult, Model, Popup};
use crate::airflow::model::common::OpenItem;
use crate::app::worker::WorkerMessage;
use popup::DagPopUp;

/// Model for the DAG panel, managing the list of DAGs and their filtering.
pub struct DagModel {
    /// Filterable table containing all DAGs and filtered view
    pub table: FilterableTable<Dag>,
    /// DAG statistics by `dag_id`
    pub dag_stats: HashMap<DagId, Vec<DagStatistic>>,
    /// Unified popup state (error, commands, or custom for this model)
    pub popup: Popup<DagPopUp>,
    /// DAG source code viewer
    pub dag_code: Option<DagCodeView>,
    ticks: u32,
    poll_tick_multiplier: u32,
    event_buffer: Vec<KeyCode>,
}

impl Default for DagModel {
    fn default() -> Self {
        Self {
            table: FilterableTable::new(),
            dag_stats: HashMap::new(),
            popup: Popup::None,
            dag_code: None,
            ticks: 0,
            poll_tick_multiplier: 10,
            event_buffer: Vec::new(),
        }
    }
}

impl DagModel {
    pub fn new(poll_tick_multiplier: u32) -> Self {
        Self {
            poll_tick_multiplier,
            ..Self::default()
        }
    }

    /// Handle model-specific popup (returns messages from popup)
    fn handle_popup(
        &mut self,
        event: &FlowrsEvent,
        ctx: &crate::app::state::NavigationContext,
    ) -> Option<Vec<WorkerMessage>> {
        let custom_popup = self.popup.custom_mut()?;
        let DagPopUp::Trigger(trigger_popup) = custom_popup;
        let (key_event, messages) = trigger_popup.update(event, ctx);
        debug!("Popup messages: {messages:?}");

        if let Some(FlowrsEvent::Key(key_event)) = &key_event {
            if matches!(
                key_event.code,
                KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q')
            ) {
                self.popup.close();
            }
        }
        Some(messages)
    }

    /// Handle dag code viewer navigation
    fn handle_dag_code_viewer(&mut self, key_code: KeyCode) -> KeyResult {
        let Some(view) = self.dag_code.as_mut() else {
            return KeyResult::Ignored;
        };
        if view.update(key_code) {
            self.dag_code = None;
        }
        KeyResult::Consumed
    }

    /// Handle model-specific keys
    fn handle_keys(&mut self, key_code: KeyCode) -> KeyResult {
        match key_code {
            KeyCode::Char('p') => {
                if let Some(dag) = self.table.current_mut() {
                    let current_state = dag.is_paused;
                    dag.is_paused = !current_state;
                    KeyResult::ConsumedWith(vec![WorkerMessage::ToggleDag {
                        dag_id: dag.dag_id.clone(),
                        is_paused: current_state,
                    }])
                } else {
                    self.popup
                        .show_error(vec!["No DAG selected to pause/resume".to_string()]);
                    KeyResult::Consumed
                }
            }
            KeyCode::Char('?') => {
                self.popup.show_commands(&DAG_COMMAND_POP_UP);
                KeyResult::Consumed
            }
            KeyCode::Enter => {
                if let Some(dag) = self.table.current() {
                    debug!("Selected dag: {}", dag.dag_id);
                    KeyResult::PassWith(vec![WorkerMessage::UpdateDagRuns {
                        dag_id: dag.dag_id.clone(),
                    }])
                } else {
                    self.popup
                        .show_error(vec!["No DAG selected to view DAG Runs".to_string()]);
                    KeyResult::Consumed
                }
            }
            KeyCode::Char('o') => {
                if let Some(dag) = self.table.current() {
                    debug!("Selected dag: {}", dag.dag_id);
                    KeyResult::PassWith(vec![WorkerMessage::OpenItem(OpenItem::Dag {
                        dag_id: dag.dag_id.clone(),
                    })])
                } else {
                    self.popup
                        .show_error(vec!["No DAG selected to open in the browser".to_string()]);
                    KeyResult::Consumed
                }
            }
            KeyCode::Char('v') => {
                if let Some(dag) = self.table.current() {
                    KeyResult::ConsumedWith(vec![WorkerMessage::GetDagCode {
                        dag_id: dag.dag_id.clone(),
                    }])
                } else {
                    self.popup
                        .show_error(vec!["No DAG selected to view code".to_string()]);
                    KeyResult::Consumed
                }
            }
            KeyCode::Char('t') => {
                if let Some(dag) = self.table.current() {
                    self.popup
                        .show_custom(DagPopUp::Trigger(TriggerDagRunPopUp::new(
                            dag.dag_id.clone(),
                        )));
                } else {
                    self.popup
                        .show_error(vec!["No DAG selected to trigger".to_string()]);
                }
                KeyResult::Consumed
            }
            _ => KeyResult::PassThrough,
        }
    }
}

impl Model for DagModel {
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
                (
                    Some(FlowrsEvent::Tick),
                    vec![WorkerMessage::UpdateDagsAndStats],
                )
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
                    .or_else(|| self.handle_dag_code_viewer(key_event.code))
                    .or_else(|| {
                        self.table
                            .handle_navigation(key_event.code, &mut self.event_buffer)
                    })
                    .or_else(|| self.handle_keys(key_event.code));

                result.into_result(event)
            }
            FlowrsEvent::Mouse | FlowrsEvent::FocusGained | FlowrsEvent::FocusLost => {
                (Some(event.clone()), vec![])
            }
        }
    }
}
