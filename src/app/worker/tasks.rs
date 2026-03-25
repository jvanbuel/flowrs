use std::sync::{Arc, Mutex};

use log::debug;

use crate::airflow::graph::TaskGraph;
use crate::airflow::traits::AirflowClient;
use crate::app::model::dagruns::popup::DagRunPopUp;
use crate::app::model::taskinstances::popup::graph::DagGraphPopup;
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

/// Handle showing the DAG graph popup from the dagrun panel.
///
/// Fetches tasks and task instances (same data as entering a dagrun),
/// then builds the graph popup and displays it on the dagrun panel.
pub async fn handle_show_dag_graph(
    app: &Arc<Mutex<App>>,
    client: &Arc<dyn AirflowClient>,
    dag_id: &str,
    dag_run_id: &str,
) {
    debug!("Fetching tasks and instances for DAG graph: {dag_id}/{dag_run_id}");

    let (tasks_result, instances_result) = tokio::join!(
        client.list_tasks(dag_id),
        client.list_task_instances(dag_id, dag_run_id),
    );

    match (tasks_result, instances_result) {
        (Ok(task_list), Ok(instance_list)) => {
            let graph = TaskGraph::from_tasks(&task_list.tasks);
            if graph.is_empty() {
                return;
            }
            let popup = DagGraphPopup::new(&graph, &instance_list.task_instances);
            let mut app = app.lock().unwrap();
            app.dagruns.popup.show_custom(DagRunPopUp::Graph(popup));
        }
        (Err(e), _) | (_, Err(e)) => {
            log::warn!("Failed to fetch data for DAG graph: {e}");
            let mut app = app.lock().unwrap();
            app.dagruns
                .popup
                .show_error(vec![format!("Failed to load DAG graph: {e}")]);
        }
    }
}
