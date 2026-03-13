use crossterm::event::KeyCode;

use crate::app::{
    events::custom::FlowrsEvent,
    model::{popup::SelectedButton, Model},
    worker::WorkerMessage,
};

use crate::airflow::model::common::{DagId, DagRunId};

pub struct ClearDagRunPopup {
    pub dag_run_ids: Vec<DagRunId>,
    pub dag_id: DagId,
    pub(crate) selected_button: SelectedButton,
}

impl ClearDagRunPopup {
    pub fn new(dag_run_ids: Vec<DagRunId>, dag_id: DagId) -> Self {
        Self {
            dag_run_ids,
            dag_id,
            selected_button: SelectedButton::default(),
        }
    }
}

impl Model for ClearDagRunPopup {
    fn update(
        &mut self,
        event: &FlowrsEvent,
        _ctx: &crate::app::state::NavigationContext,
    ) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        if let FlowrsEvent::Key(key_event) = event {
            match key_event.code {
                KeyCode::Enter => {
                    // On Enter, we always return the key event, so the parent can close the popup
                    // If Yes is selected, we also return WorkerMessages to clear the dag runs
                    if self.selected_button.is_yes() {
                        return (
                            Some(FlowrsEvent::Key(*key_event)),
                            self.dag_run_ids
                                .iter()
                                .map(|dag_run_id| WorkerMessage::ClearDagRun {
                                    dag_run_id: dag_run_id.clone(),
                                    dag_id: self.dag_id.clone(),
                                })
                                .collect(),
                        );
                    }
                    return (Some(FlowrsEvent::Key(*key_event)), vec![]);
                }
                KeyCode::Char('j' | 'k' | 'h' | 'l')
                | KeyCode::Down
                | KeyCode::Up
                | KeyCode::Left
                | KeyCode::Right => {
                    self.selected_button.toggle();
                    return (None, vec![]);
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    // On Esc, we always return the key event, so the parent can close the popup, without clearing the dag run
                    return (Some(FlowrsEvent::Key(*key_event)), vec![]);
                }
                _ => {}
            }
        }
        (Some(event.clone()), vec![])
    }
}
