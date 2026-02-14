use std::sync::{Arc, Mutex};

use futures::future::join_all;
use log::debug;

use crate::airflow::traits::AirflowClient;
use crate::app::model::popup::error::ErrorPopup;
use crate::app::state::App;

/// Handle fetching task logs for all attempts of a task instance.
///
/// `env_name` identifies which environment initiated this request, ensuring
/// results are written to the correct environment even if the active one changes.
pub async fn handle_update_task_logs(
    app: &Arc<Mutex<App>>,
    client: &Arc<dyn AirflowClient>,
    dag_id: &str,
    dag_run_id: &str,
    task_id: &str,
    task_try: u16,
    env_name: &str,
) {
    debug!("Getting logs for task: {task_id}, try number {task_try}");
    let logs =
        join_all((1..=task_try).map(|i| client.get_task_logs(dag_id, dag_run_id, task_id, i)))
            .await;

    let mut app = app.lock().unwrap();
    let mut collected_logs = Vec::new();
    for log in logs {
        match log {
            Ok(log) => {
                debug!("Got log: {log:?}");
                collected_logs.push(log);
            }
            Err(e) => {
                debug!("Error getting logs: {e}");
                app.logs.error_popup = Some(ErrorPopup::from_strings(vec![e.to_string()]));
            }
        }
    }

    // Store logs in the originating environment, not the active one
    if !collected_logs.is_empty() {
        if let Some(env) = app.environment_state.get_environment_mut(env_name) {
            env.add_task_logs(dag_id, dag_run_id, task_id, collected_logs);
        }
    }

    // Only sync panel data if this environment is still active
    if app.environment_state.is_active_environment(env_name) {
        app.sync_panel(&crate::app::state::Panel::Logs);
    }
}
