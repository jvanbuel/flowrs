use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use super::model::dagruns::popup::mark::MarkState;
use super::model::taskinstances::popup::mark::MarkState as TaskMarkState;
use super::state::App;
use crate::airflow::model::common::{DagId, DagRunId, OpenItem, TaskId};
use anyhow::Result;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinSet;

mod browser;
mod dagruns;
mod dags;
mod logs;
mod taskinstances;
mod tasks;

#[derive(Debug)]
pub struct Dispatcher {
    app: Arc<Mutex<App>>,
}

#[derive(Debug)]
pub enum WorkerMessage {
    UpdateDagsAndStats,
    ToggleDag {
        dag_id: DagId,
        is_paused: bool,
    },
    UpdateDagRuns {
        dag_id: DagId,
    },
    UpdateTaskInstances {
        dag_id: DagId,
        dag_run_id: DagRunId,
    },
    GetDagCode {
        dag_id: DagId,
    },
    GetDagParams {
        dag_id: DagId,
    },
    ClearDagRun {
        dag_run_id: DagRunId,
        dag_id: DagId,
    },
    UpdateTaskLogs {
        dag_id: DagId,
        dag_run_id: DagRunId,
        task_id: TaskId,
        task_try: u32,
    },
    MarkDagRun {
        dag_run_id: DagRunId,
        dag_id: DagId,
        status: MarkState,
    },
    ClearTaskInstance {
        task_id: TaskId,
        dag_id: DagId,
        dag_run_id: DagRunId,
    },
    MarkTaskInstance {
        task_id: TaskId,
        dag_id: DagId,
        dag_run_id: DagRunId,
        status: TaskMarkState,
    },
    TriggerDagRun {
        dag_id: DagId,
        conf: Option<serde_json::Value>,
    },
    UpdateTasks {
        dag_id: DagId,
    },
    ShowDagGraph {
        dag_id: DagId,
        dag_run_id: DagRunId,
    },
    OpenItem(OpenItem),
}

/// A message bound to the environment that was active when it was sent, so a
/// later environment switch cannot redirect it.
#[derive(Debug)]
pub struct Request {
    pub environment: Option<String>,
    pub message: WorkerMessage,
}

impl Request {
    /// Refreshes for the same data in the same environment are duplicates;
    /// the same refresh for another environment is not.
    fn dedup_key(&self) -> Option<String> {
        let key = self.message.dedup_key()?;
        Some(format!(
            "{}:{key}",
            self.environment.as_deref().unwrap_or_default()
        ))
    }
}

impl WorkerMessage {
    /// Returns a dedup key for periodic refresh messages.
    /// One-off user actions (mark, clear, trigger, toggle, etc.) return `None`
    /// so they are never deduplicated.
    pub(crate) fn dedup_key(&self) -> Option<String> {
        match self {
            Self::UpdateDagsAndStats => Some("UpdateDagsAndStats".to_string()),
            Self::UpdateDagRuns { dag_id } => Some(format!("UpdateDagRuns:{dag_id}")),
            Self::UpdateTaskInstances { dag_id, dag_run_id } => {
                Some(format!("UpdateTaskInstances:{dag_id}:{dag_run_id}"))
            }
            Self::UpdateTaskLogs {
                dag_id,
                dag_run_id,
                task_id,
                ..
            } => Some(format!("UpdateTaskLogs:{dag_id}:{dag_run_id}:{task_id}")),
            Self::UpdateTasks { dag_id } => Some(format!("UpdateTasks:{dag_id}")),
            // One-off operations should never be deduplicated
            _ => None,
        }
    }
}

impl Dispatcher {
    pub const fn new(app: Arc<Mutex<App>>) -> Self {
        Self { app }
    }

    pub async fn run(self, mut rx: Receiver<Request>) -> Result<()> {
        let mut tasks = JoinSet::new();
        let in_flight: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

        // Process messages until the channel is closed (recv returns None)
        while let Some(request) = rx.recv().await {
            let dedup_key = request.dedup_key();

            // Skip if an identical periodic request is already in flight
            if let Some(ref key) = dedup_key {
                let mut set = in_flight.lock().unwrap();
                if set.contains(key) {
                    log::debug!("Skipping duplicate in-flight message: {key}");
                    continue;
                }
                set.insert(key.clone());
            }

            // Spawn each message processing as a concurrent task
            let app = Arc::clone(&self.app);
            let in_flight = Arc::clone(&in_flight);
            tasks.spawn(async move {
                if let Err(e) = process_message(app, request).await {
                    log::error!("Error processing message: {e}");
                }
                // Remove from in-flight set when done
                if let Some(key) = dedup_key {
                    in_flight.lock().unwrap().remove(&key);
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
async fn process_message(app: Arc<Mutex<App>>, request: Request) -> Result<()> {
    let Request {
        environment,
        message,
    } = request;
    // Set loading state at the start
    {
        let mut app = app.lock().unwrap();
        app.loading = true;
    }

    // Use the client of the environment the message was sent from, so results
    // and actions go there even if the user has switched environments since.
    let client = {
        let app = app.lock().unwrap();
        environment
            .as_deref()
            .and_then(|env| app.environment_state.environments.get(env))
            .map(|env| Arc::clone(&env.client))
    };

    let Some(client) = client else {
        let mut app = app.lock().unwrap();
        app.dags
            .popup
            .show_error(vec!["No active environment selected".into()]);
        app.loading = false;
        return Ok(());
    };

    let Some(env_name) = environment else {
        let mut app = app.lock().unwrap();
        app.loading = false;
        return Ok(());
    };

    // The ticket orders this request's result against the user's pending
    // changes, so it is taken before the request starts.
    let ticket = app.lock().unwrap().environment_state.begin_fetch(&env_name);

    match message {
        // DAG operations
        WorkerMessage::UpdateDagsAndStats => {
            dags::handle_update_dags_and_stats(&app, &client, &ticket).await;
        }
        WorkerMessage::ToggleDag { dag_id, is_paused } => {
            dags::handle_toggle_dag(&app, &client, &dag_id, is_paused, &env_name).await;
        }
        WorkerMessage::GetDagCode { dag_id } => {
            dags::handle_get_dag_code(&app, &client, &dag_id).await;
        }
        WorkerMessage::GetDagParams { dag_id } => {
            dags::handle_get_dag_params(&app, &client, &dag_id, &env_name).await;
        }
        // DAG run operations
        WorkerMessage::UpdateDagRuns { dag_id, .. } => {
            dagruns::handle_update_dag_runs(&app, &client, &dag_id, &ticket).await;
        }
        WorkerMessage::ClearDagRun { dag_run_id, dag_id } => {
            dagruns::handle_clear_dag_run(&app, &client, &dag_id, &dag_run_id).await;
        }
        WorkerMessage::MarkDagRun {
            dag_run_id,
            dag_id,
            status,
        } => {
            dagruns::handle_mark_dag_run(&app, &client, &dag_id, &dag_run_id, status, &env_name)
                .await;
        }
        WorkerMessage::TriggerDagRun { dag_id, conf } => {
            dagruns::handle_trigger_dag_run(&app, &client, &dag_id, &env_name, conf).await;
        }
        // Task instance operations
        WorkerMessage::UpdateTaskInstances {
            dag_id, dag_run_id, ..
        } => {
            taskinstances::handle_update_task_instances(
                &app,
                &client,
                &dag_id,
                &dag_run_id,
                &ticket,
            )
            .await;
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
                &env_name,
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
            logs::handle_update_task_logs(
                &app,
                &client,
                &dag_id,
                &dag_run_id,
                &task_id,
                task_try,
                &ticket,
            )
            .await;
        }
        // Task operations
        WorkerMessage::UpdateTasks { dag_id } => {
            tasks::handle_update_tasks(&app, &client, &dag_id, &ticket).await;
        }
        WorkerMessage::ShowDagGraph { dag_id, dag_run_id } => {
            tasks::handle_show_dag_graph(&app, &client, &dag_id, &dag_run_id).await;
        }
        // Browser operations
        WorkerMessage::OpenItem(item) => {
            browser::handle_open_item(&app, &client, item);
        }
    }

    // Reset loading state at the end
    {
        let mut app = app.lock().unwrap();
        app.loading = false;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn refresh_for(environment: &str) -> Request {
        Request {
            environment: Some(environment.to_string()),
            message: WorkerMessage::UpdateDagsAndStats,
        }
    }

    #[test]
    fn refreshes_for_different_environments_are_not_duplicates() {
        assert_ne!(
            refresh_for("prod").dedup_key(),
            refresh_for("dev").dedup_key()
        );
        assert_eq!(
            refresh_for("prod").dedup_key(),
            refresh_for("prod").dedup_key()
        );
    }
}
