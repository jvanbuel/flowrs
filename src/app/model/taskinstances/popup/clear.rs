use crossterm::event::KeyCode;

use crate::app::{
    events::custom::FlowrsEvent,
    model::{popup::SelectedButton, Model},
    worker::WorkerMessage,
};

use crate::airflow::model::common::{DagId, DagRunId, TaskId};

#[derive(Debug)]
pub struct ClearTaskInstancePopup {
    pub dag_run_id: DagRunId,
    pub dag_id: DagId,
    /// `(task_id, map_index)` pairs to clear.
    pub task_ids: Vec<(TaskId, i64)>,
    /// Shown when the API will clear more map indices than were selected.
    pub warning: Option<String>,
    pub(crate) selected_button: SelectedButton,
}

impl ClearTaskInstancePopup {
    pub fn new(
        dag_run_id: &DagRunId,
        dag_id: &DagId,
        task_ids: Vec<(TaskId, i64)>,
        clears_single_map_index: bool,
    ) -> Self {
        let warning = (!clears_single_map_index)
            .then(|| mapped_task_names(&task_ids))
            .filter(|names| !names.is_empty())
            .map(|names| format!("Airflow 2 clears all map indices of: {names}"));
        Self {
            dag_run_id: dag_run_id.clone(),
            dag_id: dag_id.clone(),
            task_ids,
            warning,
            selected_button: SelectedButton::default(),
        }
    }
}

/// Comma-separated, de-duplicated task ids of the mapped instances in `task_ids`.
fn mapped_task_names(task_ids: &[(TaskId, i64)]) -> String {
    let mut names: Vec<&str> = task_ids
        .iter()
        .filter(|(_, map_index)| *map_index >= 0)
        .map(|(task_id, _)| task_id.as_ref())
        .collect();
    names.sort_unstable();
    names.dedup();
    names.join(", ")
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
                                .map(|(task_id, map_index)| WorkerMessage::ClearTaskInstance {
                                    dag_run_id: self.dag_run_id.clone(),
                                    dag_id: self.dag_id.clone(),
                                    task_id: task_id.clone(),
                                    map_index: *map_index,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warns_only_for_mapped_tasks_when_api_clears_all_indices() {
        let ids = vec![("a".into(), 0), ("a".into(), 1), ("b".into(), -1)];
        let popup = ClearTaskInstancePopup::new(&"r".into(), &"d".into(), ids.clone(), false);
        assert_eq!(
            popup.warning.as_deref(),
            Some("Airflow 2 clears all map indices of: a")
        );

        let popup = ClearTaskInstancePopup::new(&"r".into(), &"d".into(), ids, true);
        assert!(popup.warning.is_none());

        let unmapped = vec![("b".into(), -1)];
        let popup = ClearTaskInstancePopup::new(&"r".into(), &"d".into(), unmapped, false);
        assert!(popup.warning.is_none());
    }
}
