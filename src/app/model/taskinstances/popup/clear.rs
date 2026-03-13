use crossterm::event::KeyCode;

use crate::app::{
    events::custom::FlowrsEvent,
    model::{popup::SelectedButton, Model},
    worker::WorkerMessage,
};

use crate::airflow::model::common::{DagId, DagRunId, TaskId};

pub struct ClearTaskInstancePopup {
    pub dag_run_id: DagRunId,
    pub dag_id: DagId,
    pub task_ids: Vec<TaskId>,
    pub(crate) selected_button: SelectedButton,
}

impl ClearTaskInstancePopup {
    pub fn new(dag_run_id: &DagRunId, dag_id: &DagId, task_ids: Vec<TaskId>) -> Self {
        Self {
            dag_run_id: dag_run_id.clone(),
            dag_id: dag_id.clone(),
            task_ids,
            selected_button: SelectedButton::default(),
        }
    }
}

impl Model for ClearTaskInstancePopup {
    fn update(
        &mut self,
        event: &FlowrsEvent,
        _ctx: &crate::app::state::NavigationContext,
    ) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        if let FlowrsEvent::Key(key_event) = event {
            match key_event.code {
                KeyCode::Enter => {
                    // On Enter, we always return the key event, so the parent can close the popup
                    // If Yes is selected, we also return WorkerMessages to clear the task instances
                    if self.selected_button.is_yes() {
                        return (
                            Some(FlowrsEvent::Key(*key_event)),
                            self.task_ids
                                .iter()
                                .map(|task_id| WorkerMessage::ClearTaskInstance {
                                    dag_run_id: self.dag_run_id.clone(),
                                    dag_id: self.dag_id.clone(),
                                    task_id: task_id.clone(),
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
