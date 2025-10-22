use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::airflow::model::common::Dag;

use super::model::popup::error::ErrorPopup;
use super::model::popup::taskinstances::mark::MarkState as taskMarkState;
use super::{model::popup::dagruns::mark::MarkState, state::App};
use anyhow::Result;
use futures::future::join_all;
use log::debug;
use tokio::sync::mpsc::Receiver;

pub struct Worker {
    app: Arc<Mutex<App>>,
    rx: Receiver<WorkerMessage>,
}

#[derive(Debug)]
pub enum WorkerMessage {
    ConfigSelected(usize),
    UpdateDags,
    ToggleDag {
        dag_id: String,
        is_paused: bool,
    },
    UpdateDagRuns {
        dag_id: String,
        clear: bool,
    },
    UpdateTaskInstances {
        dag_id: String,
        dag_run_id: String,
        clear: bool,
    },
    GetDagCode {
        dag_id: String,
    },
    UpdateDagStats {
        clear: bool,
    },
    ClearDagRun {
        dag_run_id: String,
        dag_id: String,
    },
    UpdateTaskLogs {
        dag_id: String,
        dag_run_id: String,
        task_id: String,
        task_try: u16,
        clear: bool,
    },
    MarkDagRun {
        dag_run_id: String,
        dag_id: String,
        status: MarkState,
    },
    ClearTaskInstance {
        task_id: String,
        dag_id: String,
        dag_run_id: String,
    },
    MarkTaskInstance {
        task_id: String,
        dag_id: String,
        dag_run_id: String,
        status: taskMarkState,
    },
    TriggerDagRun {
        dag_id: String,
    },
    OpenItem(OpenItem),
}

#[derive(Debug)]
pub enum OpenItem {
    Config(String),
    Dag {
        dag_id: String,
    },
    DagRun {
        dag_id: String,
        dag_run_id: String,
    },
    TaskInstance {
        dag_id: String,
        dag_run_id: String,
        task_id: String,
    },
    Log {
        dag_id: String,
        dag_run_id: String,
        task_id: String,
        #[allow(dead_code)]
        task_try: u16,
    },
}

impl Worker {
    pub fn new(app: Arc<Mutex<App>>, rx_worker: Receiver<WorkerMessage>) -> Self {
        Worker { app, rx: rx_worker }
    }

    pub async fn process_message(&mut self, message: WorkerMessage) -> Result<()> {
        // Set loading state at the start
        {
            let mut app = self.app.lock().unwrap();
            app.loading = true;
        }

        // Get the active client from the environment state
        let client = {
            let app = self.app.lock().unwrap();
            app.environment_state.get_active_client()
        };

        if client.is_none() {
            // Reset loading state before returning
            let mut app = self.app.lock().unwrap();
            app.dags.error_popup = Some(ErrorPopup::from_strings(vec![
                "No active environment selected".into(),
            ]));
            app.loading = false;
            return Ok(());
        }
        let client = client.unwrap();
        match message {
            WorkerMessage::UpdateDags => {
                let dag_list = client.list_dags().await;
                let mut app = self.app.lock().unwrap();
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
            WorkerMessage::ToggleDag { dag_id, is_paused } => {
                let dag = client.toggle_dag(&dag_id, is_paused).await;
                if let Err(e) = dag {
                    let mut app = self.app.lock().unwrap();
                    app.dags.error_popup = Some(ErrorPopup::from_strings(vec![e.to_string()]));
                }
            }
            WorkerMessage::ConfigSelected(idx) => {
                self.switch_airflow_client(idx);
            }
            WorkerMessage::UpdateDagRuns { dag_id, clear: _ } => {
                let dag_runs = client.list_dagruns(&dag_id).await;
                let mut app = self.app.lock().unwrap();
                // Note: dag_id is already set in the event loop before this runs
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
                        app.dagruns.error_popup =
                            Some(ErrorPopup::from_strings(vec![e.to_string()]));
                    }
                }
            }
            WorkerMessage::UpdateTaskInstances {
                dag_id,
                dag_run_id,
                clear: _,
            } => {
                let task_instances = client.list_task_instances(&dag_id, &dag_run_id).await;
                let mut app = self.app.lock().unwrap();
                // Note: dag_id and dag_run_id are already set in the event loop before this runs
                match task_instances {
                    Ok(task_instances) => {
                        // Store task instances in the environment state
                        if let Some(env) = app.environment_state.get_active_environment_mut() {
                            for task_instance in &task_instances.task_instances {
                                env.upsert_task_instance(task_instance.clone());
                            }
                        }
                        // Sync panel data from environment state to refresh with new API data
                        app.sync_panel_data();
                    }

                    Err(e) => {
                        log::error!("Error getting task instances: {e:?}");
                        app.task_instances.error_popup =
                            Some(ErrorPopup::from_strings(vec![e.to_string()]));
                    }
                }
            }
            WorkerMessage::GetDagCode { dag_id } => {
                let current_dag: Option<Dag>;
                {
                    let app = self.app.lock().unwrap();
                    current_dag = app.environment_state.get_active_dag(&dag_id);
                }

                if let Some(current_dag) = current_dag {
                    let dag_code = client.get_dag_code(&current_dag).await;
                    let mut app = self.app.lock().unwrap();
                    match dag_code {
                        Ok(dag_code) => {
                            app.dagruns.dag_code.set_code(&dag_code);
                        }
                        Err(e) => {
                            app.dags.error_popup =
                                Some(ErrorPopup::from_strings(vec![e.to_string()]));
                        }
                    }
                } else {
                    let mut app = self.app.lock().unwrap();
                    app.dags.error_popup =
                        Some(ErrorPopup::from_strings(vec!["DAG not found".to_string()]));
                }
            }
            WorkerMessage::UpdateDagStats { clear } => {
                let dag_ids = {
                    let app = self.app.lock().unwrap();
                    let dag_ids = app
                        .environment_state
                        .get_active_dags()
                        .iter()
                        .map(|dag| dag.dag_id.clone())
                        .collect::<Vec<_>>();
                    dag_ids
                };
                let dag_ids_str: Vec<&str> =
                    dag_ids.iter().map(std::string::String::as_str).collect();
                let dag_stats = client.get_dag_stats(dag_ids_str).await;
                let mut app = self.app.lock().unwrap();
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
            WorkerMessage::ClearDagRun { dag_run_id, dag_id } => {
                debug!("Clearing dag_run: {dag_run_id}");
                let dag_run = client.clear_dagrun(&dag_id, &dag_run_id).await;
                if let Err(e) = dag_run {
                    debug!("Error clearing dag_run: {e}");
                    let mut app = self.app.lock().unwrap();
                    app.dagruns.error_popup = Some(ErrorPopup::from_strings(vec![e.to_string()]));
                }
            }
            WorkerMessage::UpdateTaskLogs {
                dag_id,
                dag_run_id,
                task_id,
                task_try,
                clear: _,
            } => {
                debug!("Getting logs for task: {task_id}, try number {task_try}");
                let logs = join_all(
                    (1..=task_try)
                        .map(|i| client.get_task_logs(&dag_id, &dag_run_id, &task_id, i))
                        .collect::<Vec<_>>(),
                )
                .await;

                // Note: dag_id, dag_run_id, and task_id are already set in the event loop before this runs

                let mut app = self.app.lock().unwrap();
                let mut collected_logs = Vec::new();
                for log in logs {
                    match log {
                        Ok(log) => {
                            debug!("Got log: {log:?}");
                            collected_logs.push(log);
                        }
                        Err(e) => {
                            debug!("Error getting logs: {e}");
                            app.logs.error_popup =
                                Some(ErrorPopup::from_strings(vec![e.to_string()]));
                        }
                    }
                }

                // Store logs in the environment state
                if !collected_logs.is_empty() {
                    if let Some(env) = app.environment_state.get_active_environment_mut() {
                        env.add_task_logs(&dag_id, &dag_run_id, &task_id, collected_logs);
                    }
                }

                // Sync panel data from environment state to refresh with new API data
                app.sync_panel_data();
            }
            WorkerMessage::MarkDagRun {
                dag_run_id,
                dag_id,
                status,
            } => {
                debug!("Marking dag_run: {dag_run_id}");
                {
                    // Update the local state before sending the request; this way, the UI will update immediately
                    let mut app = self.app.lock().unwrap();
                    app.dagruns.mark_dag_run(&dag_run_id, &status.to_string());
                }
                let dag_run = client
                    .mark_dag_run(&dag_id, &dag_run_id, &status.to_string())
                    .await;
                if let Err(e) = dag_run {
                    debug!("Error marking dag_run: {e}");
                    let mut app = self.app.lock().unwrap();
                    app.dagruns.error_popup = Some(ErrorPopup::from_strings(vec![e.to_string()]));
                }
            }
            WorkerMessage::ClearTaskInstance {
                task_id,
                dag_id,
                dag_run_id,
            } => {
                debug!("Clearing task_instance: {task_id}");
                let task_instance = client
                    .clear_task_instance(&dag_id, &dag_run_id, &task_id)
                    .await;
                if let Err(e) = task_instance {
                    debug!("Error clearing task_instance: {e}");
                    let mut app = self.app.lock().unwrap();
                    app.task_instances.error_popup =
                        Some(ErrorPopup::from_strings(vec![e.to_string()]));
                }
            }
            WorkerMessage::MarkTaskInstance {
                task_id,
                dag_id,
                dag_run_id,
                status,
            } => {
                debug!("Marking task_instance: {task_id}");
                {
                    // Update the local state before sending the request; this way, the UI will update immediately
                    let mut app = self.app.lock().unwrap();
                    app.task_instances
                        .mark_task_instance(&task_id, &status.to_string());
                }
                let task_instance = client
                    .mark_task_instance(&dag_id, &dag_run_id, &task_id, &status.to_string())
                    .await;
                if let Err(e) = task_instance {
                    debug!("Error marking task_instance: {e}");
                    let mut app = self.app.lock().unwrap();
                    app.task_instances.error_popup =
                        Some(ErrorPopup::from_strings(vec![e.to_string()]));
                }
            }
            WorkerMessage::TriggerDagRun { dag_id } => {
                debug!("Triggering dag_run: {dag_id}");
                let dag_run = client.trigger_dag_run(&dag_id, None).await;
                if let Err(e) = dag_run {
                    debug!("Error triggering dag_run: {e}");
                    let mut app = self.app.lock().unwrap();
                    app.dagruns.error_popup = Some(ErrorPopup::from_strings(vec![e.to_string()]));
                }
            }
            WorkerMessage::OpenItem(item) => {
                // For Config items, look up the endpoint from active_server instead of using the passed string
                let final_item = if let OpenItem::Config(_) = &item {
                    let app = self.app.lock().unwrap();

                    let active_server_name = app
                        .config
                        .active_server
                        .as_ref()
                        .ok_or_else(|| anyhow::anyhow!("No active server configured"))?;

                    let servers = app
                        .config
                        .servers
                        .as_ref()
                        .ok_or_else(|| anyhow::anyhow!("No servers configured"))?;

                    let server = servers
                        .iter()
                        .find(|s| &s.name == active_server_name)
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "Active server '{active_server_name}' not found in configuration"
                            )
                        })?;

                    OpenItem::Config(server.endpoint.clone())
                } else {
                    item
                };

                let url = client.build_open_url(&final_item)?;
                webbrowser::open(&url).unwrap();
            }
        }

        // Reset loading state at the end
        {
            let mut app = self.app.lock().unwrap();
            app.loading = false;
        }

        Ok(())
    }

    pub fn switch_airflow_client(&mut self, idx: usize) {
        let mut app = self.app.lock().unwrap();
        let selected_config = app.configs.filtered.items[idx].clone();
        let env_name = selected_config.name.clone();

        // Check if environment already exists, if not create it
        if !app.environment_state.environments.contains_key(&env_name) {
            if let Ok(client) = crate::airflow::client::create_client(&selected_config) {
                let env_data = crate::app::environment_state::EnvironmentData::new(client);
                app.environment_state
                    .add_environment(env_name.clone(), env_data);
            }
        }

        // Set this as the active environment
        app.environment_state
            .set_active_environment(env_name.clone());
        app.config.active_server = Some(env_name);

        // Clear the view state but NOT the environment data
        app.clear_state();

        // Sync panel data from the new environment
        app.sync_panel_data();
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            if let Some(message) = self.rx.recv().await {
                // tokio::spawn(async move {
                //     self.process_message(message).await;
                // }); //TODO: check how we can send messages to a pool of workers
                self.process_message(message).await?;
            }
        }
    }
}
