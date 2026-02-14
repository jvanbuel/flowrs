use std::sync::{Arc, Mutex};

use crate::airflow::traits::AirflowClient;
use crate::app::state::App;

/// Handle updating DAGs and their statistics from the Airflow server.
/// Fetches DAG list and stats concurrently. Stats use cached DAG IDs so both
/// requests can start immediately; new DAGs pick up stats on the next refresh.
pub async fn handle_update_dags_and_stats(app: &Arc<Mutex<App>>, client: &Arc<dyn AirflowClient>) {
    // Snapshot cached DAG IDs for the stats request (avoids holding the lock during I/O)
    let cached_dag_ids: Vec<String> = {
        let app_lock = app.lock().unwrap();
        app_lock
            .environment_state
            .get_active_dags()
            .iter()
            .map(|dag| dag.dag_id.clone())
            .collect()
    };

    // Fetch DAG list and stats concurrently
    let (dag_list_result, dag_stats_result) = tokio::join!(client.list_dags(), async {
        let refs: Vec<&str> = cached_dag_ids.iter().map(String::as_str).collect();
        client.get_dag_stats(refs).await
    });

    let mut app = app.lock().unwrap();

    // Process DAGs — full replacement evicts stale entries
    match dag_list_result {
        Ok(dag_list) => {
            if let Some(env) = app.environment_state.get_active_environment_mut() {
                env.replace_dags(dag_list.dags);
            }
        }
        Err(e) => {
            app.dags.popup.show_error(vec![e.to_string()]);
        }
    }

    // Process stats
    match dag_stats_result {
        Ok(dag_stats) => {
            if let Some(env) = app.environment_state.get_active_environment_mut() {
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

    // Sync the Dag panel from environment state
    app.sync_panel(&crate::app::state::Panel::Dag);
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
