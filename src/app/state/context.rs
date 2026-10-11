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
                // The task graph belongs to one DAG; entering another DAG
                // must not leave the previous DAG's graph in place.
                if self.nav_context.dag_id() != Some(dag_id) {
                    self.task_instances.task_graph = None;
                }
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
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use flowrs_config::FlowrsConfig;

    use crate::airflow::graph::TaskGraph;
    use crate::app::state::App;
    use crate::app::worker::WorkerMessage;

    fn app_in_environment() -> App {
        let config = FlowrsConfig::parse_toml(r#"active_server = "prod""#).unwrap();
        App::new(config)
    }

    fn enter_dag(app: &mut App, dag_id: &str) {
        app.set_context_from_message(&WorkerMessage::UpdateDagRuns {
            dag_id: dag_id.into(),
        });
    }

    fn enter_dag_run(app: &mut App, dag_id: &str) {
        // Mirrors the DagRun panel's Enter: UpdateTasks is sent before
        // UpdateTaskInstances, after the DAG was already entered.
        app.set_context_from_message(&WorkerMessage::UpdateTasks {
            dag_id: dag_id.into(),
        });
        app.set_context_from_message(&WorkerMessage::UpdateTaskInstances {
            dag_id: dag_id.into(),
            dag_run_id: "run_1".into(),
        });
    }

    #[test]
    fn entering_another_dag_drops_the_previous_task_graph() {
        let mut app = app_in_environment();
        enter_dag(&mut app, "dag_a");
        enter_dag_run(&mut app, "dag_a");
        app.task_instances.task_graph = Some(TaskGraph::default());

        enter_dag(&mut app, "dag_b");
        enter_dag_run(&mut app, "dag_b");

        assert!(app.task_instances.task_graph.is_none());
    }

    #[test]
    fn reentering_the_same_dag_keeps_its_task_graph() {
        let mut app = app_in_environment();
        enter_dag(&mut app, "dag_a");
        enter_dag_run(&mut app, "dag_a");
        app.task_instances.task_graph = Some(TaskGraph::default());

        enter_dag(&mut app, "dag_a");
        enter_dag_run(&mut app, "dag_a");

        assert!(app.task_instances.task_graph.is_some());
    }

    #[test]
    fn clearing_state_drops_the_task_graph() {
        let mut app = app_in_environment();
        enter_dag(&mut app, "dag_a");
        app.task_instances.task_graph = Some(TaskGraph::default());

        app.clear_state();

        assert!(app.task_instances.task_graph.is_none());
    }
}
