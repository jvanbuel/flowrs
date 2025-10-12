use std::sync::{Arc, Mutex};

use crate::airflow::{model::common::Dag, traits::AirflowClient as AirflowClientTrait};

use super::model::popup::error::ErrorPopup;
use super::model::popup::taskinstances::mark::MarkState as taskMarkState;
use super::{model::popup::dagruns::mark::MarkState, state::App};
use anyhow::Result;
use futures::future::join_all;
use log::debug;
use tokio::sync::mpsc::Receiver;
use url::{form_urlencoded, Url};

pub struct Worker {
    app: Arc<Mutex<App>>,
    client: Option<Arc<dyn AirflowClientTrait>>,
    rx_worker: Receiver<WorkerMessage>,
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
    /// Constructs a Worker with shared application state, an optional Airflow client, and a message receiver.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// use tokio::sync::mpsc::channel;
    /// use tokio::sync::Mutex;
    ///
    /// // Create placeholders for `App`, `WorkerMessage` and a client implementing `AirflowClientTrait`.
    /// let app = Arc::new(Mutex::new(App::default()));
    /// let (tx, rx) = channel::<WorkerMessage>(8);
    /// let client: Option<Arc<dyn AirflowClientTrait>> = None;
    ///
    /// let worker = Worker::new(app, client, rx);
    /// ```
    pub fn new(
        app: Arc<Mutex<App>>,
        client: Option<Arc<dyn AirflowClientTrait>>,
        rx_worker: Receiver<WorkerMessage>,
    ) -> Self {
        Worker {
            app,
            client,
            rx_worker,
        }
    }

    /// Processes a single `WorkerMessage`, performing the requested Airflow client operations and updating shared application state or opening external URLs.
    ///
    /// This method dispatches the provided message to the configured Airflow client (if present). It updates the `App` inside `self.app` with results or error popups, performs optimistic local updates where appropriate, and opens the web browser for navigation messages. If no client is configured it will handle `ConfigSelected` to switch clients and otherwise return immediately.
    ///
    /// # Errors
    ///
    /// Returns an error if URL construction for `OpenItem` fails or if underlying client calls propagate an error type that is converted into the returned `anyhow::Error`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tokio::runtime::Runtime;
    /// // Assume `worker` is created elsewhere and is mutable.
    /// // let mut worker = create_worker(...);
    /// // Runtime::new().unwrap().block_on(async {
    /// //     worker.process_message(WorkerMessage::UpdateDags).await.unwrap();
    /// // });
    /// ```
    pub async fn process_message(&mut self, message: WorkerMessage) -> Result<()> {
        if self.client.is_none() {
            if let WorkerMessage::ConfigSelected(idx) = message {
                self.switch_airflow_client(idx);
            };
            return Ok(());
        }
        let client = self.client.as_ref().unwrap();
        match message {
            WorkerMessage::UpdateDags => {
                let dag_list = client.list_dags().await;
                let mut app = self.app.lock().unwrap();
                match dag_list {
                    Ok(dag_list) => {
                        app.dags.all = dag_list.dags;
                        app.dags.filter_dags();
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
            WorkerMessage::UpdateDagRuns { dag_id, clear } => {
                let dag_runs = client.list_dagruns(&dag_id).await;
                let mut app = self.app.lock().unwrap();
                if clear {
                    app.dagruns.dag_id = Some(dag_id);
                }
                match dag_runs {
                    Ok(dag_runs) => {
                        app.dagruns.all = dag_runs.dag_runs;
                        app.dagruns.filter_dag_runs();
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
                clear,
            } => {
                let task_instances = client.list_task_instances(&dag_id, &dag_run_id).await;
                let mut app = self.app.lock().unwrap();
                if clear {
                    app.task_instances.dag_run_id = Some(dag_run_id);
                    app.task_instances.dag_id = Some(dag_id);
                }
                match task_instances {
                    Ok(task_instances) => {
                        app.task_instances.all = task_instances.task_instances;
                        app.task_instances.filter_task_instances();
                    }

                    Err(e) => {
                        log::error!("Error getting task instances: {:?}", e);
                        app.task_instances.error_popup =
                            Some(ErrorPopup::from_strings(vec![e.to_string()]));
                    }
                }
            }
            WorkerMessage::GetDagCode { dag_id } => {
                let current_dag: Dag;
                {
                    let app = self.app.lock().unwrap();
                    current_dag = app
                        .dags
                        .get_dag_by_id(&dag_id)
                        .expect("Dag not found")
                        .clone();
                }

                let dag_code = client.get_dag_code(&current_dag.file_token).await;
                let mut app = self.app.lock().unwrap();
                match dag_code {
                    Ok(dag_code) => {
                        app.dagruns.dag_code.code = Some(dag_code);
                    }
                    Err(e) => {
                        app.dags.error_popup = Some(ErrorPopup::from_strings(vec![e.to_string()]));
                    }
                }
            }
            WorkerMessage::UpdateDagStats { clear } => {
                let dag_ids = {
                    let app = self.app.lock().unwrap();
                    let dag_ids = app
                        .dags
                        .all
                        .iter()
                        .map(|dag| dag.dag_id.clone())
                        .collect::<Vec<_>>();
                    dag_ids
                };
                let dag_ids_str: Vec<&str> = dag_ids.iter().map(|s| s.as_str()).collect();
                let dag_stats = client.get_dag_stats(dag_ids_str).await;
                let mut app = self.app.lock().unwrap();
                if clear {
                    app.dags.dag_stats = Default::default();
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
                debug!("Clearing dag_run: {}", dag_run_id);
                let dag_run = client.clear_dagrun(&dag_id, &dag_run_id).await;
                if let Err(e) = dag_run {
                    debug!("Error clearing dag_run: {}", e);
                    let mut app = self.app.lock().unwrap();
                    app.dagruns.error_popup = Some(ErrorPopup::from_strings(vec![e.to_string()]));
                }
            }
            WorkerMessage::UpdateTaskLogs {
                dag_id,
                dag_run_id,
                task_id,
                task_try,
                clear,
            } => {
                debug!("Getting logs for task: {task_id}, try number {task_try}");
                let logs = join_all(
                    (1..=task_try)
                        .map(|i| client.get_task_logs(&dag_id, &dag_run_id, &task_id, i))
                        .collect::<Vec<_>>(),
                )
                .await;

                if clear {
                    let mut app = self.app.lock().unwrap();
                    app.logs.dag_id = Some(dag_id);
                    app.logs.dag_run_id = Some(dag_run_id);
                    app.logs.task_id = Some(task_id);
                }

                let mut app = self.app.lock().unwrap();
                app.logs.all.clear();
                for log in logs {
                    match log {
                        Ok(log) => {
                            app.logs.all.push(log);
                        }
                        Err(e) => {
                            debug!("Error getting logs: {}", e);
                            app.logs.error_popup =
                                Some(ErrorPopup::from_strings(vec![e.to_string()]));
                        }
                    }
                }
            }
            WorkerMessage::MarkDagRun {
                dag_run_id,
                dag_id,
                status,
            } => {
                debug!("Marking dag_run: {}", dag_run_id);
                {
                    // Update the local state before sending the request; this way, the UI will update immediately
                    let mut app = self.app.lock().unwrap();
                    app.dagruns.mark_dag_run(&dag_run_id, &status.to_string());
                }
                let dag_run = client
                    .mark_dag_run(&dag_id, &dag_run_id, &status.to_string())
                    .await;
                if let Err(e) = dag_run {
                    debug!("Error marking dag_run: {}", e);
                    let mut app = self.app.lock().unwrap();
                    app.dagruns.error_popup = Some(ErrorPopup::from_strings(vec![e.to_string()]));
                }
            }
            WorkerMessage::ClearTaskInstance {
                task_id,
                dag_id,
                dag_run_id,
            } => {
                debug!("Clearing task_instance: {}", task_id);
                let task_instance = client
                    .clear_task_instance(&dag_id, &dag_run_id, &task_id)
                    .await;
                if let Err(e) = task_instance {
                    debug!("Error clearing task_instance: {}", e);
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
                debug!("Marking task_instance: {}", task_id);
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
                    debug!("Error marking task_instance: {}", e);
                    let mut app = self.app.lock().unwrap();
                    app.task_instances.error_popup =
                        Some(ErrorPopup::from_strings(vec![e.to_string()]));
                }
            }
            WorkerMessage::TriggerDagRun { dag_id } => {
                debug!("Triggering dag_run: {}", dag_id);
                let dag_run = client.trigger_dag_run(&dag_id).await;
                if let Err(e) = dag_run {
                    debug!("Error triggering dag_run: {}", e);
                    let mut app = self.app.lock().unwrap();
                    app.dagruns.error_popup = Some(ErrorPopup::from_strings(vec![e.to_string()]));
                }
            }
            WorkerMessage::OpenItem(item) => {
                let endpoint = {
                    let app = self.app.lock().unwrap();
                    app.config
                        .active_server
                        .as_ref()
                        .and_then(|server_name| {
                            app.config.servers.as_ref().and_then(|servers| {
                                servers
                                    .iter()
                                    .find(|s| s.name == *server_name)
                                    .map(|s| s.endpoint.clone())
                            })
                        })
                        .unwrap_or_default()
                };
                let mut base_url = Url::parse(&endpoint)?;
                match item {
                    OpenItem::Config(config_endpoint) => {
                        base_url = config_endpoint.parse()?;
                    }
                    OpenItem::Dag { dag_id } => {
                        base_url = base_url.join(&format!("dags/{dag_id}"))?;
                    }
                    OpenItem::DagRun { dag_id, dag_run_id } => {
                        base_url = base_url.join(&format!("dags/{dag_id}/grid"))?;
                        let escaped_dag_run_id: String =
                            form_urlencoded::byte_serialize(dag_run_id.as_bytes()).collect();
                        base_url.set_query(Some(&format!("dag_run_id={escaped_dag_run_id}")));
                    }
                    OpenItem::TaskInstance {
                        dag_id,
                        dag_run_id,
                        task_id,
                    } => {
                        base_url = base_url.join(&format!("dags/{dag_id}/grid"))?;
                        let escaped_dag_run_id: String =
                            form_urlencoded::byte_serialize(dag_run_id.as_bytes()).collect();
                        base_url.set_query(Some(&format!(
                            "dag_run_id={escaped_dag_run_id}&task_id={task_id}"
                        )));
                    }
                    OpenItem::Log {
                        dag_id,
                        dag_run_id,
                        task_id,
                        task_try: _,
                    } => {
                        base_url = base_url.join(&format!("/dags/{dag_id}/grid"))?;
                        let escaped_dag_run_id: String =
                            form_urlencoded::byte_serialize(dag_run_id.as_bytes()).collect();
                        base_url.set_query(Some(&format!(
                            "dag_run_id={escaped_dag_run_id}&task_id={task_id}&tab=logs"
                        )));
                    }
                }
                webbrowser::open(base_url.as_str()).unwrap();
            }
        }
        Ok(())
    }

    /// Switches the worker's Airflow client to the configuration at the given index and resets application state.
    
    ///
    
    /// This replaces the current client with a new client created from the selected configuration, sets
    
    /// `app.config.active_server` to the selected config's name, and clears the app state.
    
    ///
    
    /// # Examples
    
    ///
    
    /// ```no_run
    
    /// // assuming `worker` is a mutable Worker with populated configs
    
    /// worker.switch_airflow_client(0);
    
    /// ```
    pub fn switch_airflow_client(&mut self, idx: usize) {
        let selected_config = self.app.lock().unwrap().configs.filtered.items[idx].clone();
        self.client = crate::airflow::client::create_client(selected_config.clone()).ok();

        let mut app = self.app.lock().unwrap();
        app.config.active_server = Some(selected_config.name.clone());
        app.clear_state();
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            if let Some(message) = self.rx_worker.recv().await {
                // tokio::spawn(async move {
                //     self.process_message(message).await;
                // }); //TODO: check how we can send messages to a pool of workers
                self.process_message(message).await?;
            }
        }
    }
}