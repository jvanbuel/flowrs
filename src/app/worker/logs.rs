use std::sync::{Arc, Mutex};

use futures::future::join_all;
use log::debug;

use crate::airflow::client::FlowrsClient;
use crate::airflow::model::common::{DagId, DagRunId, TaskId};
use crate::app::model::popup::error::ErrorPopup;
use crate::app::state::environment_state::{FetchTicket, Fetched};
use crate::app::state::App;

/// Handle fetching task logs for all attempts of a task instance.
///
/// The ticket names the environment that initiated this request, so results
/// are written there even if the active environment changes meanwhile.
pub async fn handle_update_task_logs(
    app: &Arc<Mutex<App>>,
    client: &Arc<FlowrsClient>,
    dag_id: &DagId,
    dag_run_id: &DagRunId,
    task_id: &TaskId,
    task_try: u32,
    ticket: &FetchTicket,
) {
    debug!("Getting logs for task: {task_id}, try number {task_try}");
    let logs =
        join_all((1..=task_try).map(|i| client.get_task_logs(dag_id, dag_run_id, task_id, i)))
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

    if !collected_logs.is_empty() {
        app.apply_fetched(
            ticket,
            Fetched::TaskLogs {
                dag_id: dag_id.clone(),
                dag_run_id: dag_run_id.clone(),
                task_id: task_id.clone(),
                logs: collected_logs,
            },
        );
    }
}
