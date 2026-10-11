use std::sync::{Arc, Mutex};

use log::debug;

use crate::airflow::client::FlowrsClient;
use crate::airflow::model::common::{DagId, DagRunId, DagRunState};
use crate::app::model::dagruns::popup::mark::MarkState;
use crate::app::state::environment_state::{FetchTicket, Fetched, Patch};
use crate::app::state::App;

/// Handle updating the list of DAG runs for a specific DAG.
///
/// The ticket names the environment that initiated this request, so results
/// are written there even if the active environment changes meanwhile.
pub async fn handle_update_dag_runs(
    app: &Arc<Mutex<App>>,
    client: &Arc<FlowrsClient>,
    dag_id: &DagId,
    ticket: &FetchTicket,
) {
    let dag_runs = client.list_dagruns(dag_id).await;

    let mut app = app.lock().unwrap();
    match dag_runs {
        Ok(dag_runs) => app.apply_fetched(
            ticket,
            Fetched::DagRuns {
                dag_id: dag_id.clone(),
                dag_runs: dag_runs.dag_runs,
            },
        ),
        Err(e) => app.dagruns.popup.show_error(vec![e.to_string()]),
    }
}

/// Handle clearing a DAG run (resets all task instances).
pub async fn handle_clear_dag_run(
    app: &Arc<Mutex<App>>,
    client: &Arc<FlowrsClient>,
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

/// Handle marking a DAG run with a new state (success/failed). The new state
/// shows at once and is undone if the server rejects it.
pub async fn handle_mark_dag_run(
    app: &Arc<Mutex<App>>,
    client: &Arc<FlowrsClient>,
    dag_id: &DagId,
    dag_run_id: &DagRunId,
    status: MarkState,
) {
    debug!("Marking dag_run: {dag_run_id}");
    let patch = Patch::DagRunState {
        dag_id: dag_id.clone(),
        dag_run_id: dag_run_id.clone(),
        state: DagRunState::from(&status),
    };
    app.lock().unwrap().patch(patch.clone());
    let result = client
        .mark_dag_run(dag_id, dag_run_id, &status.to_string())
        .await;
    let mut app = app.lock().unwrap();
    app.settle(&patch, result.is_ok());
    if let Err(e) = result {
        debug!("Error marking dag_run: {e}");
        app.dagruns.popup.show_error(vec![e.to_string()]);
    }
}

/// Handle triggering a new DAG run.
pub async fn handle_trigger_dag_run(
    app: &Arc<Mutex<App>>,
    client: &Arc<FlowrsClient>,
    dag_id: &DagId,
    env_name: &str,
    conf: Option<serde_json::Value>,
) {
    debug!("Triggering dag_run: {dag_id}");
    let dag_run = client.trigger_dag_run(dag_id, None, conf).await;
    match dag_run {
        Ok(()) => {
            // Refresh the dag runs list to show the newly triggered run
            let ticket = app.lock().unwrap().environment_state.begin_fetch(env_name);
            handle_update_dag_runs(app, client, dag_id, &ticket).await;
        }
        Err(e) => {
            debug!("Error triggering dag_run: {e}");
            let mut app = app.lock().unwrap();
            app.dagruns.popup.show_error(vec![e.to_string()]);
        }
    }
}
