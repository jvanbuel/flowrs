use std::sync::{Arc, Mutex};

use futures::future::join_all;
use log::debug;

use crate::airflow::client::FlowrsClient;
use crate::airflow::model::common::{DagId, DagRunId, TaskId};
use crate::app::model::popup::error::ErrorPopup;
use crate::app::state::App;

/// Handle fetching task logs for all attempts of a task instance.
///
/// `env_name` identifies which environment initiated this request, ensuring
/// results are written to the correct environment even if the active one changes.
#[expect(
    clippy::too_many_arguments,
    reason = "each argument is one part of the task instance key; a struct would only wrap them"
)]
pub async fn handle_update_task_logs(
    app: &Arc<Mutex<App>>,
    client: &Arc<FlowrsClient>,
    dag_id: &DagId,
    dag_run_id: &DagRunId,
    task_id: &TaskId,
    map_index: i64,
    task_try: u32,
    env_name: &str,
) {
    debug!("Getting logs for task: {task_id}[{map_index}], try number {task_try}");
    let logs = join_all(
        (1..=task_try).map(|i| client.get_task_logs(dag_id, dag_run_id, task_id, map_index, i)),
    )
    .await;

    // Collect logs and errors outside the lock
    let mut collected_logs = Vec::new();
    let mut errors = Vec::new();
    for log in logs {
        match log {
            Ok(log) => {
                debug!("Got log: {log:?}");
                collected_logs.push(log);
            }
            Err(e) => {
                debug!("Error getting logs: {e}");
                errors.push(e.to_string());
            }
        }
    }

    let mut app = app.lock().unwrap();

    if !errors.is_empty() {
        app.logs.error_popup = Some(ErrorPopup::from_strings(errors));
    }

    // Store logs in the originating environment, not the active one
    if !collected_logs.is_empty() {
        if let Some(env) = app.environment_state.environments.get_mut(env_name) {
            env.replace_task_logs(dag_id, dag_run_id, task_id, map_index, collected_logs);
        }
    }

    // Only sync panel data if this environment is still active
    if app.environment_state.active_environment.as_deref() == Some(env_name) {
        app.sync_panel(&crate::app::state::Panel::Logs);
    }
}
