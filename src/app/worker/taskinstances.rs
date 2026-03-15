use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use futures::future::join_all;
use log::debug;

use crate::airflow::model::common::{DagId, DagRunId, GanttData, TaskId, TaskInstanceState};
use crate::airflow::traits::AirflowClient;
use crate::app::model::taskinstances::popup::mark::MarkState;
use crate::app::state::App;

/// Handle updating the list of task instances for a specific DAG run.
///
/// `env_name` identifies which environment initiated this request, ensuring
/// results are written to the correct environment even if the active one changes.
///
/// Fetches task instances, then fetches detailed try history for any retried
/// tasks, builds the Gantt chart from complete data, and stores everything
/// atomically under a single lock.
pub async fn handle_update_task_instances(
    app: &Arc<Mutex<App>>,
    client: &Arc<dyn AirflowClient>,
    dag_id: &DagId,
    dag_run_id: &DagRunId,
    env_name: &str,
) {
    // 1. Fetch task instances (no lock)
    let task_instances = match client.list_task_instances(dag_id, dag_run_id).await {
        Ok(list) => list.task_instances,
        Err(e) => {
            log::error!("Error getting task instances: {e:?}");
            let mut app = app.lock().unwrap();
            app.task_instances.popup.show_error(vec![e.to_string()]);
            return;
        }
    };

    // 2. Identify retried tasks and fetch their tries concurrently (no lock)
    let mut seen = HashSet::new();
    let retried_task_ids: Vec<&TaskId> = task_instances
        .iter()
        .filter(|ti| ti.try_number > 1 && seen.insert(&ti.task_id))
        .map(|ti| &ti.task_id)
        .collect();

    let tries_results = join_all(
        retried_task_ids
            .iter()
            .map(|task_id| client.list_task_instance_tries(dag_id, dag_run_id, task_id)),
    )
    .await;

    // 3. Build gantt from task instances, then overlay detailed tries (no lock)
    let mut gantt = GanttData::from_task_instances(&task_instances);
    for (task_id, result) in retried_task_ids.iter().zip(tries_results) {
        match result {
            Ok(tries) => {
                gantt.update_tries(task_id, tries);
            }
            Err(e) => {
                log::warn!("Failed to fetch tries for task {task_id}: {e}");
            }
        }
    }

    // 4. Store everything atomically under a single lock
    let mut app = app.lock().unwrap();
    if let Some(env) = app.environment_state.environments.get_mut(env_name) {
        env.replace_task_instances(dag_id, dag_run_id, task_instances);
    }
    if app.environment_state.active_environment.as_deref() == Some(env_name) {
        app.sync_panel(&crate::app::state::Panel::TaskInstance);
        app.task_instances.gantt_data = gantt;
    }
}

/// Handle clearing a task instance (resets it to be re-run).
pub async fn handle_clear_task_instance(
    app: &Arc<Mutex<App>>,
    client: &Arc<dyn AirflowClient>,
    dag_id: &DagId,
    dag_run_id: &DagRunId,
    task_id: &TaskId,
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
    dag_id: &DagId,
    dag_run_id: &DagRunId,
    task_id: &TaskId,
    status: MarkState,
) {
    debug!("Marking task_instance: {task_id}");
    {
        // Update the local state before sending the request; this way, the UI will update immediately
        let mut app = app.lock().unwrap();
        app.task_instances
            .mark_task_instance(task_id, TaskInstanceState::from(&status));
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
