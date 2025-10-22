use std::sync::{Arc, Mutex};

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use events::{custom::FlowrsEvent, generator::EventGenerator};
use log::debug;
use model::Model;
use ratatui::{prelude::Backend, Terminal};
use state::{App, Panel};
use worker::{Worker, WorkerMessage};

use crate::{airflow::client::create_client, ui::draw_ui};

pub mod environment_state;
pub mod events;
pub mod model;
pub mod state;
pub mod worker;

pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: Arc<Mutex<App>>) -> Result<()> {
    let mut events = EventGenerator::new(200);
    let ui_app = app.clone();
    let worker_app = app.clone();

    let (tx_worker, rx_worker) = tokio::sync::mpsc::channel::<WorkerMessage>(100);

    log::info!("Initializing environment state");
    {
        let mut app = app.lock().unwrap();

        // Clone servers to avoid borrow checker issues
        let servers = app.config.servers.clone();
        let active_server_name = app.config.active_server.clone();

        // Initialize all environments with their clients
        if let Some(servers) = servers {
            for server_config in servers {
                if let Ok(client) = create_client(&server_config) {
                    let env_data = environment_state::EnvironmentData::new(client);
                    app.environment_state
                        .add_environment(server_config.name.clone(), env_data);
                } else {
                    log::error!(
                        "Failed to create client for server '{}'; skipping",
                        server_config.name
                    );
                }
            }
        }

        // Set the active environment if one was configured
        if let Some(active_server_name) = active_server_name {
            app.environment_state
                .set_active_environment(active_server_name);
        }
    }

    log::info!("Spawning worker");
    tokio::spawn(async move { Worker::new(worker_app, rx_worker).run().await });

    loop {
        terminal.draw(|f| {
            debug!("Drawing UI");
            draw_ui(f, &ui_app);
        })?;

        if let Some(event) = events.next().await {
            // First handle panel specific events, and send messages to the event channel
            let (fall_through_event, messages) = {
                let mut app = app.lock().unwrap();
                match app.active_panel {
                    Panel::Config => app.configs.update(&event),
                    Panel::Dag => app.dags.update(&event),
                    Panel::DAGRun => app.dagruns.update(&event),
                    Panel::TaskInstance => app.task_instances.update(&event),
                    Panel::Logs => app.logs.update(&event),
                }
            };

            // Process messages and sync cached data immediately
            for message in &messages {
                // Set context IDs and sync cached data before worker processes the message
                {
                    let mut app = app.lock().unwrap();
                    match message {
                        WorkerMessage::UpdateDagRuns { dag_id, clear } => {
                            if *clear {
                                app.dagruns.dag_id = Some(dag_id.clone());
                                // Sync cached data immediately
                                app.dagruns.all = app.environment_state.get_active_dag_runs(dag_id);
                                app.dagruns.filter_dag_runs();
                            }
                        }
                        WorkerMessage::UpdateTaskInstances {
                            dag_id,
                            dag_run_id,
                            clear,
                        } => {
                            if *clear {
                                app.task_instances.dag_id = Some(dag_id.clone());
                                app.task_instances.dag_run_id = Some(dag_run_id.clone());
                                // Sync cached data immediately
                                app.task_instances.all = app
                                    .environment_state
                                    .get_active_task_instances(dag_id, dag_run_id);
                                app.task_instances.filter_task_instances();
                            }
                        }
                        WorkerMessage::UpdateTaskLogs {
                            dag_id,
                            dag_run_id,
                            task_id,
                            clear,
                            ..
                        } => {
                            if *clear {
                                app.logs.dag_id = Some(dag_id.clone());
                                app.logs.dag_run_id = Some(dag_run_id.clone());
                                app.logs.task_id = Some(task_id.clone());
                                // Sync cached data immediately
                                app.logs.all = app
                                    .environment_state
                                    .get_active_task_logs(dag_id, dag_run_id, task_id);
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Now send messages to worker for async processing
            for message in messages {
                if let Err(e) = tx_worker.send(message).await {
                    log::error!("Failed to send message to worker: {e}");
                }
            }
            if fall_through_event.is_none() {
                continue;
            }

            // We do this so that when a user switches config,
            // it does not show the previous DAGs (because the Enter event falls through before the existing DAGs are cleared).
            // Not very mindful, not very demure.
            if let Some(FlowrsEvent::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            })) = fall_through_event
            {
                let mut app = app.lock().unwrap();
                if let Panel::Config = app.active_panel {
                    app.ticks = 0;
                }
            }

            // then handle generic events
            let mut app = app.lock().unwrap();
            if let Some(FlowrsEvent::Tick) = fall_through_event {
                app.ticks += 1;
                app.throbber_state.calc_next();
            }
            if let FlowrsEvent::Key(key) = event {
                // Handle exit key events
                if key.modifiers == KeyModifiers::CONTROL {
                    if let KeyCode::Char('c' | 'd') = key.code {
                        return Ok(());
                    }
                }
                // Handle other key events
                match key.code {
                    KeyCode::Char('q') => {
                        app.config.write_to_file()?;
                        return Ok(());
                    }
                    KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                        app.next_panel();
                        app.sync_panel_data();
                    }
                    KeyCode::Esc | KeyCode::Left | KeyCode::Char('h') => {
                        app.previous_panel();
                        app.sync_panel_data();
                    }
                    _ => {}
                }
            }
        }
    }
}
