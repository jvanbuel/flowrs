use std::vec;

use crossterm::event::KeyCode;

use crate::app::events::custom::FlowrsEvent;
use crate::app::worker::WorkerMessage;

pub struct ClearDagRunPopup {
    pub dag_run_id: String,
    pub dag_id: String,
    pub confirm: bool,
}

impl ClearDagRunPopup {
    pub fn new(dag_run_id: String, dag_id: String) -> Self {
        ClearDagRunPopup {
            dag_run_id,
            dag_id,
            confirm: false,
        }
    }

    pub fn update(&mut self, event: &FlowrsEvent) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        if let FlowrsEvent::Key(key_event) = event {
            match key_event.code {
                KeyCode::Enter => {
                    // On Enter, we always return the key event, so the parent can close the popup
                    // If the confirm flag is set, we also return a WorkerMessage to clear the dag run
                    if self.confirm {
                        return (
                            Some(FlowrsEvent::Key(*key_event)),
                            vec![WorkerMessage::ClearDagRun {
                                dag_run_id: self.dag_run_id.clone(),
                                dag_id: self.dag_id.clone(),
                            }],
                        );
                    } else {
                        return (Some(FlowrsEvent::Key(*key_event)), vec![]);
                    }
                }
                KeyCode::Char('j')
                | KeyCode::Down
                | KeyCode::Char('k')
                | KeyCode::Up
                | KeyCode::Char('h')
                | KeyCode::Left
                | KeyCode::Char('l')
                | KeyCode::Right => {
                    // For any movement vim key, we toggle the confirm flag, and we consume the event
                    self.confirm = !self.confirm;
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
