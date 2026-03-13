use crossterm::event::KeyCode;
use strum::Display;

use crate::airflow::model::common::{DagId, DagRunId, DagRunState};
use crate::app::{events::custom::FlowrsEvent, model::Model, worker::WorkerMessage};

pub struct MarkDagRunPopup {
    pub dag_id: DagId,
    pub status: MarkState,
    pub marked: Vec<DagRunId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Display)]
pub enum MarkState {
    #[strum(to_string = "success")]
    Success,
    #[strum(to_string = "failed")]
    Failed,
    #[strum(to_string = "queued")]
    Queued,
}

impl From<&MarkState> for DagRunState {
    fn from(state: &MarkState) -> Self {
        match state {
            MarkState::Success => Self::Success,
            MarkState::Failed => Self::Failed,
            MarkState::Queued => Self::Queued,
        }
    }
}

impl MarkDagRunPopup {
    pub const fn new(marked: Vec<DagRunId>, dag_id: DagId) -> Self {
        Self {
            dag_id,
            status: MarkState::Success,
            marked,
        }
    }

    pub const fn next_state(&mut self) {
        self.status = match self.status {
            MarkState::Success => MarkState::Failed,
            MarkState::Failed => MarkState::Queued,
            MarkState::Queued => MarkState::Success,
        };
    }

    pub const fn previous_state(&mut self) {
        self.status = match self.status {
            MarkState::Success => MarkState::Queued,
            MarkState::Failed => MarkState::Success,
            MarkState::Queued => MarkState::Failed,
        };
    }
}

impl Model for MarkDagRunPopup {
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
                            .map(|i| WorkerMessage::MarkDagRun {
                                dag_run_id: i.clone(),
                                dag_id: self.dag_id.clone(),
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
