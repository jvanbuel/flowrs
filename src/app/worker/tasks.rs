use std::sync::{Arc, Mutex};

use log::debug;

use crate::airflow::graph::TaskGraph;
use crate::airflow::traits::AirflowClient;
use crate::app::state::App;

/// Handle fetching task definitions and building the task graph
pub async fn handle_update_tasks(
    app: &Arc<Mutex<App>>,
    client: &Arc<dyn AirflowClient>,
    dag_id: &str,
) {
    debug!("Fetching tasks for DAG: {dag_id}");

    match client.list_tasks(dag_id).await {
        Ok(task_list) => {
            let graph = TaskGraph::from_tasks(&task_list.tasks);
            debug!("Built task graph with {} tasks", task_list.tasks.len());

            let mut app = app.lock().unwrap();
            app.task_instances.task_graph = Some(graph);
            app.task_instances.sort_task_instances();
            app.task_instances.table.apply_filter();
        }
        Err(e) => {
            // Graceful degradation: log warning but don't show error popup
            log::warn!("Failed to fetch tasks for {dag_id}: {e}");
            // Task instances will remain unsorted
        }
    }
}
