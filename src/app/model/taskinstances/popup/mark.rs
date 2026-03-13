use crossterm::event::KeyCode;
use strum::Display;

use crate::airflow::model::common::{DagId, DagRunId, TaskId, TaskInstanceState};
use crate::app::{events::custom::FlowrsEvent, model::Model, worker::WorkerMessage};

pub struct MarkTaskInstancePopup {
    pub dag_id: DagId,
    pub dag_run_id: DagRunId,
    pub status: MarkState,
    pub marked: Vec<TaskId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Display)]
pub enum MarkState {
    #[strum(to_string = "success")]
    Success,
    #[strum(to_string = "failed")]
    Failed,
    #[strum(to_string = "skipped")]
    Skipped,
}

impl From<&MarkState> for TaskInstanceState {
    fn from(state: &MarkState) -> Self {
        match state {
            MarkState::Success => Self::Success,
            MarkState::Failed => Self::Failed,
            MarkState::Skipped => Self::Skipped,
        }
    }
}

impl MarkTaskInstancePopup {
    pub fn new(marked: Vec<TaskId>, dag_id: &DagId, dag_run_id: &DagRunId) -> Self {
        Self {
            dag_id: dag_id.clone(),
            status: MarkState::Success,
            marked,
            dag_run_id: dag_run_id.clone(),
        }
    }

    pub const fn next_state(&mut self) {
        self.status = match self.status {
            MarkState::Success => MarkState::Failed,
            MarkState::Failed => MarkState::Skipped,
            MarkState::Skipped => MarkState::Success,
        };
    }

    pub const fn previous_state(&mut self) {
        self.status = match self.status {
            MarkState::Success => MarkState::Skipped,
            MarkState::Failed => MarkState::Success,
            MarkState::Skipped => MarkState::Failed,
        };
    }
}

impl Model for MarkTaskInstancePopup {
    fn update(
        &mut self,
        event: &FlowrsEvent,
        _ctx: &crate::app::state::NavigationContext,
    ) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        if let FlowrsEvent::Key(key_event) = event {
            match key_event.code {
                KeyCode::Enter => {
                    // On Enter, we always return the key event, so the parent can close the popup
                    return (
                        Some(FlowrsEvent::Key(*key_event)),
                        self.marked
                            .iter()
                            .map(|i| WorkerMessage::MarkTaskInstance {
                                dag_run_id: self.dag_run_id.clone(),
                                dag_id: self.dag_id.clone(),
                                task_id: i.clone(),
                                status: self.status.clone(),
                            })
                            .collect(),
                    );
                }
                KeyCode::Char('j' | 'h') | KeyCode::Down | KeyCode::Left => {
                    self.previous_state();
                    return (None, vec![]);
                }
                KeyCode::Char('k' | 'l') | KeyCode::Up | KeyCode::Right => {
                    self.next_state();
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
