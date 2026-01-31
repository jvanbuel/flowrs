use std::sync::{Arc, Mutex};

use super::model::popup::dagruns::mark::MarkState;
use super::model::popup::taskinstances::mark::MarkState as TaskMarkState;
use super::state::App;
use anyhow::Result;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinSet;

mod browser;
mod config;
mod dagruns;
mod dags;
mod logs;
mod taskinstances;
mod tasks;

pub struct Dispatcher {
    app: Arc<Mutex<App>>,
}

#[derive(Debug)]
pub enum WorkerMessage {
    ConfigSelected(usize),
    UpdateDagsAndStats,
    ToggleDag {
        dag_id: String,
        is_paused: bool,
    },
    UpdateDagRuns {
        dag_id: String,
    },
    UpdateTaskInstances {
        dag_id: String,
        dag_run_id: String,
    },
    GetDagCode {
        dag_id: String,
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
        status: TaskMarkState,
    },
    TriggerDagRun {
        dag_id: String,
    },
    UpdateTasks {
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

impl Dispatcher {
    pub const fn new(app: Arc<Mutex<App>>) -> Self {
        Self { app }
    }

    pub async fn run(self, mut rx: Receiver<WorkerMessage>) -> Result<()> {
        let mut tasks = JoinSet::new();

        // Process messages until the channel is closed (recv returns None)
        while let Some(message) = rx.recv().await {
            // Spawn each message processing as a concurrent task
            // This allows multiple API requests to run in parallel
            let app = self.app.clone();
            tasks.spawn(async move {
                if let Err(e) = process_message(app, message).await {
                    log::error!("Error processing message: {e}");
                }
            });
        }

        // Channel closed, await all spawned tasks to finish for graceful shutdown
        while let Some(result) = tasks.join_next().await {
            if let Err(e) = result {
                log::error!("Task panicked during shutdown: {e}");
            }
        }

        Ok(())
    }
}

/// Processes a single worker message. This function is spawned as an ephemeral task
/// by the Dispatcher, allowing multiple API requests to run in parallel.
async fn process_message(app: Arc<Mutex<App>>, message: WorkerMessage) -> Result<()> {
    // Set loading state at the start
    {
        let mut app = app.lock().unwrap();
        app.loading = true;
    }

    // Handle messages that don't require an active client first
    if let WorkerMessage::ConfigSelected(idx) = message {
        return config::handle_config_selected(&app, idx);
    }

    // Get the active client from the environment state
    let client = {
        let app = app.lock().unwrap();
        app.environment_state.get_active_client()
    };

    let Some(client) = client else {
        let mut app = app.lock().unwrap();
        app.dags
            .popup
            .show_error(vec!["No active environment selected".into()]);
        app.loading = false;
        return Ok(());
    };

    match message {
        WorkerMessage::ConfigSelected(_) => {
            unreachable!("ConfigSelected should be handled before client check");
        }
        // DAG operations
        WorkerMessage::UpdateDagsAndStats => {
            dags::handle_update_dags_and_stats(&app, &client).await;
        }
        WorkerMessage::ToggleDag { dag_id, is_paused } => {
            dags::handle_toggle_dag(&app, &client, &dag_id, is_paused).await;
        }
        WorkerMessage::GetDagCode { dag_id } => {
            dags::handle_get_dag_code(&app, &client, &dag_id).await;
        }
        // DAG run operations
        WorkerMessage::UpdateDagRuns { dag_id, .. } => {
            dagruns::handle_update_dag_runs(&app, &client, &dag_id).await;
        }
        WorkerMessage::ClearDagRun { dag_run_id, dag_id } => {
            dagruns::handle_clear_dag_run(&app, &client, &dag_id, &dag_run_id).await;
        }
        WorkerMessage::MarkDagRun {
            dag_run_id,
            dag_id,
            status,
        } => {
            dagruns::handle_mark_dag_run(&app, &client, &dag_id, &dag_run_id, status).await;
        }
        WorkerMessage::TriggerDagRun { dag_id } => {
            dagruns::handle_trigger_dag_run(&app, &client, &dag_id).await;
        }
        // Task instance operations
        WorkerMessage::UpdateTaskInstances {
            dag_id, dag_run_id, ..
        } => {
            taskinstances::handle_update_task_instances(&app, &client, &dag_id, &dag_run_id).await;
        }
        WorkerMessage::ClearTaskInstance {
            task_id,
            dag_id,
            dag_run_id,
        } => {
            taskinstances::handle_clear_task_instance(
                &app,
                &client,
                &dag_id,
                &dag_run_id,
                &task_id,
            )
            .await;
        }
        WorkerMessage::MarkTaskInstance {
            task_id,
            dag_id,
            dag_run_id,
            status,
        } => {
            taskinstances::handle_mark_task_instance(
                &app,
                &client,
                &dag_id,
                &dag_run_id,
                &task_id,
                status,
            )
            .await;
        }
        // Log operations
        WorkerMessage::UpdateTaskLogs {
            dag_id,
            dag_run_id,
            task_id,
            task_try,
            ..
        } => {
            logs::handle_update_task_logs(&app, &client, &dag_id, &dag_run_id, &task_id, task_try)
                .await;
        }
        // Task operations
        WorkerMessage::UpdateTasks { dag_id } => {
            tasks::handle_update_tasks(&app, &client, &dag_id).await;
        }
        // Browser operations
        WorkerMessage::OpenItem(item) => {
            browser::handle_open_item(&app, &client, item)?;
        }
    }

    // Reset loading state at the end
    {
        let mut app = app.lock().unwrap();
        app.loading = false;
    }

    Ok(())
}
