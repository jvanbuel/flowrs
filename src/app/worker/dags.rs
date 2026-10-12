use std::sync::{Arc, Mutex};

use log::warn;

use crate::airflow::client::FlowrsClient;
use crate::airflow::model::common::{DagId, DagStatsResponse};
use crate::app::model::dagruns::popup::trigger::TriggerDagRunPopUp;
use crate::app::model::dagruns::popup::DagRunPopUp;
use crate::app::model::dagruns::DagCodeView;
use crate::app::model::dags::popup::DagPopUp;
use crate::app::navigation::Level;
use crate::app::state::environment_state::{FetchTicket, Fetched, Patch};
use crate::app::state::App;

/// Handle updating DAGs and their statistics from the Airflow server.
/// On cold start (empty cache), fetches DAGs first then stats sequentially so
/// stats use fresh IDs. On warm cache, fetches both concurrently using cached
/// DAG IDs; new DAGs pick up stats on the next refresh.
///
/// The ticket names the environment that initiated this request, so results
/// are written there even if the active environment changes meanwhile.
pub async fn handle_update_dags_and_stats(
    app: &Arc<Mutex<App>>,
    client: &Arc<FlowrsClient>,
    ticket: &FetchTicket,
) {
    // Snapshot cached DAG IDs from the originating environment for the stats request
    let cached_dag_ids: Vec<DagId> = {
        let app_lock = app.lock().unwrap();
        app_lock
            .environment_state
            .environments
            .get(ticket.environment())
            .map(|env| env.dags.iter().map(|dag| dag.dag_id.clone()).collect())
            .unwrap_or_default()
    };

    if cached_dag_ids.is_empty() {
        // Cold start: show DAGs as soon as they arrive, then fetch stats with
        // their fresh IDs
        let dag_ids: Vec<DagId> = match client.list_dags().await {
            Ok(dag_list) => {
                let ids = dag_list.dags.iter().map(|d| d.dag_id.clone()).collect();
                app.lock()
                    .unwrap()
                    .apply_fetched(ticket, Fetched::Dags(dag_list.dags));
                ids
            }
            Err(e) => {
                app.lock()
                    .unwrap()
                    .dags
                    .popup
                    .show_error(vec![e.to_string()]);
                return;
            }
        };
        if !dag_ids.is_empty() {
            let refs: Vec<&str> = dag_ids.iter().map(AsRef::as_ref).collect();
            let dag_stats_result = client.get_dag_stats(refs).await;
            apply_dag_stats(app, ticket, dag_stats_result);
        }
    } else {
        // Warm cache: fetch DAG list and stats concurrently using cached IDs
        let (dag_list_result, dag_stats_result) = tokio::join!(client.list_dags(), async {
            let refs: Vec<&str> = cached_dag_ids.iter().map(AsRef::as_ref).collect();
            client.get_dag_stats(refs).await
        });
        match dag_list_result {
            Ok(dag_list) => {
                app.lock()
                    .unwrap()
                    .apply_fetched(ticket, Fetched::Dags(dag_list.dags));
            }
            Err(e) => app
                .lock()
                .unwrap()
                .dags
                .popup
                .show_error(vec![e.to_string()]),
        }
        apply_dag_stats(app, ticket, dag_stats_result);
    }
}

fn apply_dag_stats(
    app: &Arc<Mutex<App>>,
    ticket: &FetchTicket,
    result: anyhow::Result<DagStatsResponse>,
) {
    match result {
        Ok(dag_stats) => {
            let stats = dag_stats
                .dags
                .into_iter()
                .map(|dag| (DagId::from(dag.dag_id), dag.stats))
                .collect();
            app.lock()
                .unwrap()
                .apply_fetched(ticket, Fetched::DagStats(stats));
        }
        Err(e) => log::error!("Failed to fetch dag stats: {e}"),
    }
}

/// Handle toggling the paused state of a DAG. The change shows at once and is
/// undone if the server rejects it.
pub async fn handle_toggle_dag(
    app: &Arc<Mutex<App>>,
    client: &Arc<FlowrsClient>,
    dag_id: &DagId,
    is_paused: bool,
    env_name: &str,
) {
    let patch = Patch::DagPaused {
        dag_id: dag_id.clone(),
        is_paused: !is_paused,
    };
    app.lock().unwrap().patch(env_name, patch.clone());
    let result = client.toggle_dag(dag_id, is_paused).await;
    let mut app = app.lock().unwrap();
    app.settle(env_name, &patch, result.is_ok());
    if let Err(e) = result {
        app.dags.popup.show_error(vec![e.to_string()]);
    }
}

/// Handle fetching the DAG source code.
pub async fn handle_get_dag_code(
    app: &Arc<Mutex<App>>,
    client: &Arc<FlowrsClient>,
    dag_id: &DagId,
) {
    let current_dag = {
        let app_lock = app.lock().unwrap();
        app_lock
            .environment_state
            .get_active_environment()
            .and_then(|env| env.dags.iter().find(|d| d.dag_id == *dag_id).cloned())
    };

    if let Some(current_dag) = current_dag {
        let dag_code = client.get_dag_code(&current_dag).await;
        let mut app = app.lock().unwrap();
        match dag_code {
            Ok(dag_code) => {
                let view = Some(DagCodeView::new(&dag_code));
                match app.navigation.level() {
                    Level::Dags => app.dags.dag_code = view,
                    Level::DagRuns => app.dagruns.dag_code = view,
                    _ => {}
                }
            }
            Err(e) => match app.navigation.level() {
                Level::DagRuns => app.dagruns.popup.show_error(vec![e.to_string()]),
                _ => app.dags.popup.show_error(vec![e.to_string()]),
            },
        }
    } else {
        let mut app = app.lock().unwrap();
        let error = vec!["DAG not found".to_string()];
        match app.navigation.level() {
            Level::DagRuns => app.dagruns.popup.show_error(error),
            _ => app.dags.popup.show_error(error),
        }
    }
}

/// Fetch a fresh copy of a DAG's param schema, then open the trigger popup
/// with it on the panel that asked. The schema is re-fetched on every popup
/// open so changes on the Airflow side show up; the cached copy is only the
/// fallback when the fetch fails.
pub async fn handle_get_dag_params(
    app: &Arc<Mutex<App>>,
    client: &Arc<FlowrsClient>,
    dag_id: &DagId,
    env_name: &str,
) {
    match client.get_dag_params(dag_id).await {
        Ok(params) => {
            let mut app = app.lock().unwrap();
            if let Some(env) = app.environment_state.environments.get_mut(env_name) {
                match params {
                    Some(params) => env.update_dag_params(dag_id, params),
                    // The DAG no longer has params: drop any stale schema.
                    None => {
                        env.dag_params.remove(dag_id);
                    }
                }
            }
        }
        Err(e) => warn!("Failed to fetch dag params for {dag_id}: {e}"),
    }

    let mut app = app.lock().unwrap();
    let params = app
        .environment_state
        .environments
        .get(env_name)
        .and_then(|env| env.dag_params.get(dag_id).cloned());
    let popup = TriggerDagRunPopUp::new(dag_id.clone(), params.as_deref());
    match app.navigation.level() {
        Level::DagRuns => app.dagruns.popup.show_custom(DagRunPopUp::Trigger(popup)),
        _ => app.dags.popup.show_custom(DagPopUp::Trigger(popup)),
    }
}
