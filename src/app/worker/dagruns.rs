use std::sync::{Arc, Mutex};

use log::{debug, warn};

use crate::airflow::model::common::{DagId, DagRunId, DagRunState};
use crate::airflow::traits::AirflowClient;
use crate::app::model::dagruns::popup::mark::MarkState;
use crate::app::state::App;

/// Handle updating the list of DAG runs for a specific DAG.
///
/// `env_name` identifies which environment initiated this request, ensuring
/// results are written to the correct environment even if the active one changes.
pub async fn handle_update_dag_runs(
    app: &Arc<Mutex<App>>,
    client: &Arc<dyn AirflowClient>,
    dag_id: &DagId,
    env_name: &str,
) {
    let (dag_runs, dag_params) =
        tokio::join!(client.list_dagruns(dag_id), client.get_dag_params(dag_id));
    let mut app = app.lock().unwrap();
    match dag_runs {
        Ok(dag_runs) => {
            // Replace DAG runs in the originating environment, not the active one
            if let Some(env) = app.environment_state.environments.get_mut(env_name) {
                env.replace_dag_runs(dag_id, dag_runs.dag_runs);
                // Cache dag params alongside dag runs (failure is non-fatal)
                match dag_params {
                    Ok(Some(params)) => env.update_dag_params(dag_id, params),
                    Ok(None) => {}
                    Err(e) => warn!("Failed to fetch dag params for {dag_id}: {e}"),
                }
            }
            // Only sync panel data if this environment is still active
            if app.environment_state.active_environment.as_deref() == Some(env_name) {
                app.sync_panel(&crate::app::state::Panel::DAGRun);
            }
        }
        Err(e) => {
            app.dagruns.popup.show_error(vec![e.to_string()]);
        }
    }
}

/// Handle clearing a DAG run (resets all task instances).
pub async fn handle_clear_dag_run(
    app: &Arc<Mutex<App>>,
    client: &Arc<dyn AirflowClient>,
    dag_id: &DagId,
    dag_run_id: &DagRunId,
) {
    debug!("Clearing dag_run: {dag_run_id}");
    let dag_run = client.clear_dagrun(dag_id, dag_run_id).await;
    if let Err(e) = dag_run {
        debug!("Error clearing dag_run: {e}");
        let mut app = app.lock().unwrap();
        app.dagruns.popup.show_error(vec![e.to_string()]);
    }
}

/// Handle marking a DAG run with a new state (success/failed).
pub async fn handle_mark_dag_run(
    app: &Arc<Mutex<App>>,
    client: &Arc<dyn AirflowClient>,
    dag_id: &DagId,
    dag_run_id: &DagRunId,
    status: MarkState,
) {
    debug!("Marking dag_run: {dag_run_id}");
    {
        // Update the local state before sending the request; this way, the UI will update immediately
        let mut app = app.lock().unwrap();
        app.dagruns
            .mark_dag_run(dag_run_id, DagRunState::from(&status));
    }
    let dag_run = client
        .mark_dag_run(dag_id, dag_run_id, &status.to_string())
        .await;
    if let Err(e) = dag_run {
        debug!("Error marking dag_run: {e}");
        let mut app = app.lock().unwrap();
        app.dagruns.popup.show_error(vec![e.to_string()]);
    }
}

/// Handle triggering a new DAG run.
pub async fn handle_trigger_dag_run(
    app: &Arc<Mutex<App>>,
    client: &Arc<dyn AirflowClient>,
    dag_id: &DagId,
    env_name: &str,
    conf: Option<serde_json::Value>,
) {
    debug!("Triggering dag_run: {dag_id}");
    let dag_run = client.trigger_dag_run(dag_id, None, conf).await;
    match dag_run {
        Ok(()) => {
            // Refresh the dag runs list to show the newly triggered run
            handle_update_dag_runs(app, client, dag_id, env_name).await;
        }
        Err(e) => {
            debug!("Error triggering dag_run: {e}");
            let mut app = app.lock().unwrap();
            app.dagruns.popup.show_error(vec![e.to_string()]);
        }
    }
}
