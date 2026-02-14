use std::sync::{Arc, Mutex};

use crate::airflow::traits::AirflowClient;
use crate::app::state::App;

/// Handle updating DAGs and their statistics from the Airflow server.
/// Fetches DAGs first, then fetches stats for all DAG IDs in parallel.
///
/// `env_name` identifies which environment initiated this request, ensuring
/// results are written to the correct environment even if the active one changes.
pub async fn handle_update_dags_and_stats(
    app: &Arc<Mutex<App>>,
    client: &Arc<dyn AirflowClient>,
    env_name: &str,
) {
    // First, fetch DAGs
    let dag_list_result = client.list_dags().await;

    // Collect DAG IDs for stats query
    let dag_ids: Vec<String> = if let Ok(dag_list) = &dag_list_result {
        dag_list.dags.iter().map(|dag| dag.dag_id.clone()).collect()
    } else {
        // If DAG list failed, try to use cached DAG IDs from the originating environment
        let app_lock = app.lock().unwrap();
        app_lock
            .environment_state
            .get_environment(env_name)
            .map(|env| env.dags.keys().cloned().collect())
            .unwrap_or_default()
    };

    // Fetch stats for all DAGs
    let dag_ids_refs: Vec<&str> = dag_ids.iter().map(String::as_str).collect();
    let dag_stats_result = client.get_dag_stats(dag_ids_refs).await;

    let mut app = app.lock().unwrap();

    // Process DAGs - write to the originating environment, not the active one
    match dag_list_result {
        Ok(dag_list) => {
            if let Some(env) = app.environment_state.get_environment_mut(env_name) {
                for dag in &dag_list.dags {
                    env.upsert_dag(dag.clone());
                }
            }
        }
        Err(e) => {
            app.dags.popup.show_error(vec![e.to_string()]);
        }
    }

    // Process stats - write to the originating environment
    match dag_stats_result {
        Ok(dag_stats) => {
            if let Some(env) = app.environment_state.get_environment_mut(env_name) {
                for dag_stats in dag_stats.dags {
                    env.update_dag_stats(&dag_stats.dag_id, dag_stats.stats);
                }
            }
        }
        Err(e) => {
            // Don't overwrite existing error popup, just log
            log::error!("Failed to fetch dag stats: {e}");
        }
    }

    // Only sync panel data if this environment is still the active one,
    // otherwise we'd overwrite the UI with stale data from a different server
    if app.environment_state.is_active_environment(env_name) {
        app.sync_panel_data();
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
        app_lock.environment_state.get_active_dag(dag_id)
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
