use std::sync::{Arc, Mutex};

use log::debug;

use crate::airflow::traits::AirflowClient;
use crate::app::model::popup::taskinstances::mark::MarkState;
use crate::app::state::App;

/// Handle updating the list of task instances for a specific DAG run.
pub async fn handle_update_task_instances(
    app: &Arc<Mutex<App>>,
    client: &Arc<dyn AirflowClient>,
    dag_id: &str,
    dag_run_id: &str,
) {
    let task_instances = client.list_task_instances(dag_id, dag_run_id).await;
    let mut app = app.lock().unwrap();
    match task_instances {
        Ok(task_instances) => {
            // Store task instances in the environment state
            if let Some(env) = app.environment_state.get_active_environment_mut() {
                for task_instance in &task_instances.task_instances {
                    env.upsert_task_instance(task_instance.clone());
                }
            }
            // Sync panel data from environment state to refresh with new API data
            app.sync_panel_data();
        }
        Err(e) => {
            log::error!("Error getting task instances: {e:?}");
            app.task_instances.popup.show_error(vec![e.to_string()]);
        }
    }
}

/// Handle clearing a task instance (resets it to be re-run).
pub async fn handle_clear_task_instance(
    app: &Arc<Mutex<App>>,
    client: &Arc<dyn AirflowClient>,
    dag_id: &str,
    dag_run_id: &str,
    task_id: &str,
) {
    debug!("Clearing task_instance: {task_id}");
    let task_instance = client
        .clear_task_instance(dag_id, dag_run_id, task_id)
        .await;
    if let Err(e) = task_instance {
        debug!("Error clearing task_instance: {e}");
        let mut app = app.lock().unwrap();
        app.task_instances.popup.show_error(vec![e.to_string()]);
    }
}

/// Handle marking a task instance with a new state (success/failed).
pub async fn handle_mark_task_instance(
    app: &Arc<Mutex<App>>,
    client: &Arc<dyn AirflowClient>,
    dag_id: &str,
    dag_run_id: &str,
    task_id: &str,
    status: MarkState,
) {
    debug!("Marking task_instance: {task_id}");
    {
        // Update the local state before sending the request; this way, the UI will update immediately
        let mut app = app.lock().unwrap();
        app.task_instances
            .mark_task_instance(task_id, &status.to_string());
    }
    let task_instance = client
        .mark_task_instance(dag_id, dag_run_id, task_id, &status.to_string())
        .await;
    if let Err(e) = task_instance {
        debug!("Error marking task_instance: {e}");
        let mut app = app.lock().unwrap();
        app.task_instances.popup.show_error(vec![e.to_string()]);
    }
}
