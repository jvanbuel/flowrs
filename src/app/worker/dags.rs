use std::sync::{Arc, Mutex};

use crate::airflow::traits::AirflowClient;
use crate::app::state::App;

/// Handle updating DAGs and their statistics from the Airflow server.
/// On cold start (empty cache), fetches DAGs first then stats sequentially so
/// stats use fresh IDs. On warm cache, fetches both concurrently using cached
/// DAG IDs; new DAGs pick up stats on the next refresh.
///
/// `env_name` identifies which environment initiated this request, ensuring
/// results are written to the correct environment even if the active one changes.
pub async fn handle_update_dags_and_stats(
    app: &Arc<Mutex<App>>,
    client: &Arc<dyn AirflowClient>,
    env_name: &str,
) {
    // Snapshot cached DAG IDs from the originating environment for the stats request
    let cached_dag_ids: Vec<String> = {
        let app_lock = app.lock().unwrap();
        app_lock
            .environment_state
            .environments
            .get(env_name)
            .map(|env| env.dags.iter().map(|dag| dag.dag_id.clone()).collect())
            .unwrap_or_default()
    };

    if cached_dag_ids.is_empty() {
        // Cold start: fetch DAGs first, then stats with fresh IDs
        let dag_list_result = client.list_dags().await;

        let dag_ids: Vec<String> = {
            let mut app = app.lock().unwrap();
            match dag_list_result {
                Ok(dag_list) => {
                    let ids: Vec<String> = dag_list.dags.iter().map(|d| d.dag_id.clone()).collect();
                    if let Some(env) = app.environment_state.get_environment_mut(env_name) {
                        env.replace_dags(dag_list.dags);
                    }
                    ids
                }
                Err(e) => {
                    app.dags.popup.show_error(vec![e.to_string()]);
                    vec![]
                }
            }
        };

        if !dag_ids.is_empty() {
            let refs: Vec<&str> = dag_ids.iter().map(String::as_str).collect();
            match client.get_dag_stats(refs).await {
                Ok(dag_stats) => {
                    let mut app = app.lock().unwrap();
                    if let Some(env) = app.environment_state.get_environment_mut(env_name) {
                        for dag_stats in dag_stats.dags {
                            env.update_dag_stats(&dag_stats.dag_id, dag_stats.stats);
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to fetch dag stats: {e}");
                }
            }
        }
    } else {
        // Warm cache: fetch DAG list and stats concurrently using cached IDs
        let (dag_list_result, dag_stats_result) = tokio::join!(client.list_dags(), async {
            let refs: Vec<&str> = cached_dag_ids.iter().map(String::as_str).collect();
            client.get_dag_stats(refs).await
        });

        let mut app = app.lock().unwrap();

        match dag_list_result {
            Ok(dag_list) => {
                if let Some(env) = app.environment_state.get_environment_mut(env_name) {
                    env.replace_dags(dag_list.dags);
                }
            }
            Err(e) => {
                app.dags.popup.show_error(vec![e.to_string()]);
            }
        }

        match dag_stats_result {
            Ok(dag_stats) => {
                if let Some(env) = app.environment_state.get_environment_mut(env_name) {
                    for dag_stats in dag_stats.dags {
                        env.update_dag_stats(&dag_stats.dag_id, dag_stats.stats);
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to fetch dag stats: {e}");
            }
        }
    }

    // Only sync panel data if this environment is still the active one,
    // otherwise we'd overwrite the UI with stale data from a different server
    let mut app = app.lock().unwrap();
    if app.environment_state.is_active_environment(env_name) {
        app.sync_panel(&crate::app::state::Panel::Dag);
    }
}

/// Handle toggling the paused state of a DAG.
pub async fn handle_toggle_dag(
    app: &Arc<Mutex<App>>,
    client: &Arc<dyn AirflowClient>,
    dag_id: &str,
    is_paused: bool,
) {
    let dag = client.toggle_dag(dag_id, is_paused).await;
    if let Err(e) = dag {
        let mut app = app.lock().unwrap();
        app.dags.popup.show_error(vec![e.to_string()]);
    }
}

/// Handle fetching the DAG source code.
pub async fn handle_get_dag_code(
    app: &Arc<Mutex<App>>,
    client: &Arc<dyn AirflowClient>,
    dag_id: &str,
) {
    let current_dag = {
        let app_lock = app.lock().unwrap();
        app_lock
            .environment_state
            .get_active_environment()
            .and_then(|env| env.dags.iter().find(|d| d.dag_id == dag_id).cloned())
    };

    if let Some(current_dag) = current_dag {
        let dag_code = client.get_dag_code(&current_dag).await;
        let mut app = app.lock().unwrap();
        match dag_code {
            Ok(dag_code) => {
                app.dagruns.dag_code.set_code(&dag_code);
            }
            Err(e) => {
                app.dags.popup.show_error(vec![e.to_string()]);
            }
        }
    } else {
        let mut app = app.lock().unwrap();
        app.dags.popup.show_error(vec!["DAG not found".to_string()]);
    }
}
