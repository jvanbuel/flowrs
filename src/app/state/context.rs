use crate::app::worker::WorkerMessage;

use super::{App, NavigationContext};

impl App {
    /// Update the centralized navigation context from a `WorkerMessage`.
    pub fn set_context_from_message(&mut self, message: &WorkerMessage) {
        let env = match self.nav_context.environment() {
            Some(e) => e.clone(),
            Option::None => return,
        };

        match message {
            WorkerMessage::UpdateDagRuns { dag_id } => {
                self.nav_context = NavigationContext::Dag {
                    environment: env,
                    dag_id: dag_id.clone(),
                };
            }
            WorkerMessage::UpdateTaskInstances { dag_id, dag_run_id } => {
                self.nav_context = NavigationContext::DagRun {
                    environment: env,
                    dag_id: dag_id.clone(),
                    dag_run_id: dag_run_id.clone(),
                };
            }
            WorkerMessage::UpdateTaskLogs {
                dag_id,
                dag_run_id,
                task_id,
                task_try,
            } => {
                let is_new_context = self.nav_context.dag_id() != Some(dag_id)
                    || self.nav_context.dag_run_id() != Some(dag_run_id)
                    || self.nav_context.task_id() != Some(task_id)
                    || self.nav_context.task_try() != Some(*task_try);
                self.nav_context = NavigationContext::Task {
                    environment: env,
                    dag_id: dag_id.clone(),
                    dag_run_id: dag_run_id.clone(),
                    task_id: task_id.clone(),
                    task_try: *task_try,
                };
                if is_new_context {
                    self.logs.current = 0;
                    self.logs.reset_scroll();
                }
            }
            WorkerMessage::UpdateTasks { dag_id } if self.nav_context.dag_id() != Some(dag_id) => {
                self.task_instances.task_graph = None;
            }
            _ => {}
        }
    }
}
