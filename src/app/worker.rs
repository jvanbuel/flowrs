use std::sync::{Arc, Mutex};

use crate::airflow::model::dag;
use crate::airflow::{client::AirFlowClient, model::dag::Dag};

use super::model::popup::taskinstances::mark::MarkState as taskMarkState;
use super::{model::popup::dagruns::mark::MarkState, state::App};
use futures::future::join_all;
use log::debug;
use ratatui::text::Line;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use syntect_tui::into_span;
use tokio::sync::mpsc::Receiver;

pub struct Worker<'a> {
    app: Arc<Mutex<App<'a>>>,
    client: AirFlowClient,
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
    GetTaskLogs {
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
        status: taskMarkState,
    },
    TriggerDagRun {
        dag_id: String,
    },
}

impl<'a> Worker<'a> {
    pub fn new(
        app: Arc<Mutex<App<'a>>>,
        client: AirFlowClient,
        rx_worker: Receiver<WorkerMessage>,
    ) -> Self {
        Worker {
            app,
            client,
            rx_worker,
        }
    }

    pub async fn process_message(&mut self, message: WorkerMessage) {
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
                let mut app = self.app.lock().unwrap();
                self.client = AirFlowClient::from(app.configs.filtered.items[idx].clone());
                *app = App::new(app.config.clone()).unwrap();
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
                let task_instances = self.client.list_task_instances(&dag_id, &dag_run_id).await;
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
                        app.dagruns.dag_code.code = Some(dag_code);
                        
                    }
                    Err(e) => app.dags.errors.push(e),
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
                let dag_stats = self.client.get_dag_stats(dag_ids_str).await;
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
                    Err(e) => app.dags.errors.push(e),
                }
            }
            WorkerMessage::ClearDagRun { dag_run_id, dag_id } => {
                debug!("Clearing dag_run: {}", dag_run_id);
                let dag_run = self.client.clear_dagrun(&dag_id, &dag_run_id).await;
                if let Err(e) = dag_run {
                    debug!("Error clearing dag_run: {}", e);
                    let mut app = self.app.lock().unwrap();
                    app.dagruns.errors.push(e);
                }
            }
            WorkerMessage::GetTaskLogs {
                dag_id,
                dag_run_id,
                task_id,
                task_try,
            } => {
                debug!("Getting logs for task: {task_id}, try number {task_try}");
                let logs = join_all(
                    (1..=task_try)
                        .map(|i| self.client.get_task_logs(&dag_id, &dag_run_id, &task_id, i))
                        .collect::<Vec<_>>(),
                )
                .await;

                let mut app = self.app.lock().unwrap();
                app.logs.all.clear();
                for log in logs {
                    match log {
                        Ok(log) => {
                            app.logs.all.push(log);
                        }
                        Err(e) => {
                            debug!("Error getting logs: {}", e);
                            app.logs.errors.push(e);
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
                let dag_run = self
                    .client
                    .mark_dag_run(&dag_id, &dag_run_id, &status.to_string())
                    .await;
                if let Err(e) = dag_run {
                    debug!("Error marking dag_run: {}", e);
                    let mut app = self.app.lock().unwrap();
                    app.dagruns.errors.push(e);
                }
            }
            WorkerMessage::ClearTaskInstance {
                task_id,
                dag_id,
                dag_run_id,
            } => {
                debug!("Clearing task_instance: {}", task_id);
                let task_instance = self
                    .client
                    .clear_task_instance(&dag_id, &dag_run_id, &task_id)
                    .await;
                if let Err(e) = task_instance {
                    debug!("Error clearing task_instance: {}", e);
                    let mut app = self.app.lock().unwrap();
                    app.task_instances.errors.push(e);
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
                let task_instance = self
                    .client
                    .mark_task_instance(&dag_id, &dag_run_id, &task_id, &status.to_string())
                    .await;
                if let Err(e) = task_instance {
                    debug!("Error marking task_instance: {}", e);
                    let mut app = self.app.lock().unwrap();
                    app.task_instances.errors.push(e);
                }
            }
            WorkerMessage::TriggerDagRun { dag_id } => {
                debug!("Triggering dag_run: {}", dag_id);
                let dag_run = self.client.trigger_dag_run(&dag_id).await;
                if let Err(e) = dag_run {
                    debug!("Error triggering dag_run: {}", e);
                    let mut app = self.app.lock().unwrap();
                    app.dagruns.errors.push(e);
                }
            }
        }
    }

    pub async fn run(&mut self) {
        loop {
            if let Some(message) = self.rx_worker.recv().await {
                // tokio::spawn(async move {
                //     self.process_message(message).await;
                // }); //TODO: check how we can send messages to a pool of workers
                self.process_message(message).await;
            }
        }
    }
}

fn code_to_lines(dag_code: &str) -> Vec<Line> {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let syntax = ps.find_syntax_by_extension("py").unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    let mut lines: Vec<Line> = vec![];
    for line in LinesWithEndings::from(dag_code) {
        // LinesWithEndings enables use of newlines mode
        let line_spans: Vec<ratatui::prelude::Span> = h
            .highlight_line(line, &ps)
            .unwrap()
            .into_iter()
            .filter_map(|segment| into_span(segment).ok())
            .collect::<Vec<ratatui::prelude::Span>>();
        lines.push(Line::from(line_spans));
    }
    lines
}
