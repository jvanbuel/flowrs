use std::sync::{Arc, Mutex};

use crate::airflow::client::AirFlowClient;

use super::state::App;
use tokio::sync::mpsc::Receiver;

pub struct Worker {
    app: Arc<Mutex<App>>,
    client: AirFlowClient,
    rx_worker: Receiver<WorkerMessage>,
}

pub enum WorkerMessage {
    UpdateDags,
    ToggleDag { dag_id: String, is_paused: bool },
    ConfigSelected(usize),
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
                }
            }
        }
    }
}
