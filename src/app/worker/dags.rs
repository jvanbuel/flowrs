use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::airflow::traits::AirflowClient;
use crate::app::model::popup::error::ErrorPopup;
use crate::app::state::App;

/// Handle updating the list of DAGs from the Airflow server.
pub async fn handle_update_dags(app: &Arc<Mutex<App>>, client: &Arc<dyn AirflowClient>) {
    let dag_list = client.list_dags().await;
    let mut app = app.lock().unwrap();
    match dag_list {
        Ok(dag_list) => {
            // Store DAGs in the environment state
            if let Some(env) = app.environment_state.get_active_environment_mut() {
                for dag in &dag_list.dags {
                    env.upsert_dag(dag.clone());
                }
            }
            // Sync panel data from environment state
            app.sync_panel_data();
        }
        Err(e) => {
            app.dags.error_popup = Some(ErrorPopup::from_strings(vec![e.to_string()]));
        }
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
        app.dags.error_popup = Some(ErrorPopup::from_strings(vec![e.to_string()]));
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
                app.dags.error_popup = Some(ErrorPopup::from_strings(vec![e.to_string()]));
            }
        }
    } else {
        let mut app = app.lock().unwrap();
        app.dags.error_popup = Some(ErrorPopup::from_strings(vec!["DAG not found".to_string()]));
    }
}

/// Handle updating DAG statistics (run counts by state).
pub async fn handle_update_dag_stats(
    app: &Arc<Mutex<App>>,
    client: &Arc<dyn AirflowClient>,
    clear: bool,
) {
    let dag_ids = {
        let app_lock = app.lock().unwrap();
        app_lock
            .environment_state
            .get_active_dags()
            .iter()
            .map(|dag| dag.dag_id.clone())
            .collect::<Vec<_>>()
    };
    let dag_ids_str: Vec<&str> = dag_ids.iter().map(std::string::String::as_str).collect();
    let dag_stats = client.get_dag_stats(dag_ids_str).await;

    let mut app = app.lock().unwrap();
    if clear {
        app.dags.dag_stats = HashMap::default();
    }
    match dag_stats {
        Ok(dag_stats) => {
            for dag_stats in dag_stats.dags {
                app.dags.dag_stats.insert(dag_stats.dag_id, dag_stats.stats);
            }
        }
        Err(e) => {
            app.dags.error_popup = Some(ErrorPopup::from_strings(vec![e.to_string()]));
        }
    }
}
