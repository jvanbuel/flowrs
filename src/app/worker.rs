use std::sync::{Arc, Mutex};

use crate::airflow::{client::AirFlowClient, model::dag::Dag};

use super::state::App;
use tokio::sync::mpsc::Receiver;

pub struct Worker {
    app: Arc<Mutex<App>>,
    client: AirFlowClient,
    rx_worker: Receiver<WorkerMessage>,
}

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
}

impl Worker {
    pub fn new(
        app: Arc<Mutex<App>>,
        client: AirFlowClient,
        rx_worker: Receiver<WorkerMessage>,
    ) -> Self {
        Worker {
            app,
            client,
            rx_worker,
        }
    }

    pub async fn run(&mut self) {
        loop {
            if let Some(message) = self.rx_worker.recv().await {
                match message {
                    WorkerMessage::UpdateDags => {
                        let dag_list = self.client.list_dags().await;
                        let mut app = self.app.lock().unwrap();
                        match dag_list {
                            Ok(dag_list) => {
                                app.dags.all = dag_list.dags;
                                app.dags.filter_dags();
                            }
                            Err(e) => app.dags.errors.push(e),
                        }
                    }
                    WorkerMessage::ToggleDag { dag_id, is_paused } => {
                        let dag = self.client.toggle_dag(&dag_id, is_paused).await;
                        if let Err(e) = dag {
                            let mut app = self.app.lock().unwrap();
                            app.dags.errors.push(e);
                        }
                    }
                    WorkerMessage::ConfigSelected(idx) => {
                        let app = self.app.lock().unwrap();
                        self.client = AirFlowClient::from(app.configs.filtered.items[idx].clone());
                    }
                    WorkerMessage::UpdateDagRuns { dag_id, clear } => {
                        let dag_runs = self.client.list_dagruns(&dag_id).await;
                        let mut app = self.app.lock().unwrap();
                        if clear {
                            app.dagruns.dag_id = Some(dag_id);
                        }
                        match dag_runs {
                            Ok(dag_runs) => {
                                app.dagruns.all = dag_runs.dag_runs;
                                app.dagruns.filter_dag_runs();
                            }
                            Err(e) => app.dagruns.errors.push(e),
                        }
                    }
                    WorkerMessage::UpdateTaskInstances {
                        dag_id,
                        dag_run_id,
                        clear,
                    } => {
                        let task_instances =
                            self.client.list_task_instances(&dag_id, &dag_run_id).await;
                        let mut app = self.app.lock().unwrap();
                        if clear {
                            app.task_instances.dag_run_id = Some(dag_run_id);
                        }
                        match task_instances {
                            Ok(task_instances) => {
                                app.task_instances.all = task_instances.task_instances;
                                app.task_instances.filter_task_instances();
                            }
                            Err(e) => app.task_instances.errors.push(e),
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

                        let dag_code = self.client.get_dag_code(&current_dag.file_token).await;
                        let mut app = self.app.lock().unwrap();
                        match dag_code {
                            Ok(dag_code) => {
                                app.dagruns.dag_code = Some(dag_code);
                            }
                            Err(e) => app.dags.errors.push(e),
                        }
                    }
                }
            }
        }
    }
}
