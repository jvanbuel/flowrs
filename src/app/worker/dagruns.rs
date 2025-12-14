use std::sync::{Arc, Mutex};

use log::debug;

use crate::airflow::traits::AirflowClient;
use crate::app::model::popup::dagruns::mark::MarkState;
use crate::app::model::popup::error::ErrorPopup;
use crate::app::state::App;

/// Handle updating the list of DAG runs for a specific DAG.
pub async fn handle_update_dag_runs(
    app: &Arc<Mutex<App>>,
    client: &Arc<dyn AirflowClient>,
    dag_id: &str,
) {
    let dag_runs = client.list_dagruns(dag_id).await;
    let mut app = app.lock().unwrap();
    match dag_runs {
        Ok(dag_runs) => {
            // Store DAG runs in the environment state
            if let Some(env) = app.environment_state.get_active_environment_mut() {
                for dag_run in &dag_runs.dag_runs {
                    env.upsert_dag_run(dag_run.clone());
                }
            }
            // Sync panel data from environment state to refresh with new API data
            app.sync_panel_data();
        }
        Err(e) => {
            app.dagruns.error_popup = Some(ErrorPopup::from_strings(vec![e.to_string()]));
        }
    }
}

/// Handle clearing a DAG run (resets all task instances).
pub async fn handle_clear_dag_run(
    app: &Arc<Mutex<App>>,
    client: &Arc<dyn AirflowClient>,
    dag_id: &str,
    dag_run_id: &str,
) {
    debug!("Clearing dag_run: {dag_run_id}");
    let dag_run = client.clear_dagrun(dag_id, dag_run_id).await;
    if let Err(e) = dag_run {
        debug!("Error clearing dag_run: {e}");
        let mut app = app.lock().unwrap();
        app.dagruns.error_popup = Some(ErrorPopup::from_strings(vec![e.to_string()]));
    }
}

/// Handle marking a DAG run with a new state (success/failed).
pub async fn handle_mark_dag_run(
    app: &Arc<Mutex<App>>,
    client: &Arc<dyn AirflowClient>,
    dag_id: &str,
    dag_run_id: &str,
    status: MarkState,
) {
    debug!("Marking dag_run: {dag_run_id}");
    {
        // Update the local state before sending the request; this way, the UI will update immediately
        let mut app = app.lock().unwrap();
        app.dagruns.mark_dag_run(dag_run_id, &status.to_string());
    }
    let dag_run = client
        .mark_dag_run(dag_id, dag_run_id, &status.to_string())
        .await;
    if let Err(e) = dag_run {
        debug!("Error marking dag_run: {e}");
        let mut app = app.lock().unwrap();
        app.dagruns.error_popup = Some(ErrorPopup::from_strings(vec![e.to_string()]));
    }
}

/// Handle triggering a new DAG run.
pub async fn handle_trigger_dag_run(
    app: &Arc<Mutex<App>>,
    client: &Arc<dyn AirflowClient>,
    dag_id: &str,
) {
    debug!("Triggering dag_run: {dag_id}");
    let dag_run = client.trigger_dag_run(dag_id, None).await;
    if let Err(e) = dag_run {
        debug!("Error triggering dag_run: {e}");
        let mut app = app.lock().unwrap();
        app.dagruns.error_popup = Some(ErrorPopup::from_strings(vec![e.to_string()]));
    }
}
