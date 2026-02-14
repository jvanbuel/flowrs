use std::sync::{Arc, Mutex};

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use events::{custom::FlowrsEvent, generator::EventGenerator};
use log::debug;
use model::Model;
use ratatui::{prelude::Backend, Terminal};
use state::{App, Panel};
use worker::{Dispatcher, WorkerMessage};

use crate::{airflow::client::create_client, ui::draw_ui};

pub mod environment_state;
pub mod events;
pub mod model;
pub mod state;
pub mod worker;

pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: Arc<Mutex<App>>) -> Result<()>
where
    <B as Backend>::Error: 'static + std::marker::Send + std::marker::Sync,
{
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
                        .environments
                        .insert(server_config.name.clone(), env_data);
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

    log::info!("Spawning dispatcher");
    tokio::spawn(async move { Dispatcher::new(worker_app).run(rx_worker).await });

    loop {
        terminal.draw(|f| {
            debug!("Drawing UI");
            draw_ui(f, &ui_app);
        })?;

        if let Some(event) = events.next().await {
            // Handle focus changes first
            match &event {
                FlowrsEvent::FocusGained => {
                    app.lock().unwrap().focused = true;
                    continue;
                }
                FlowrsEvent::FocusLost => {
                    app.lock().unwrap().focused = false;
                    continue;
                }
                _ => {}
            }

            // Skip tick processing when unfocused (no automatic refreshes)
            if matches!(&event, FlowrsEvent::Tick) && !app.lock().unwrap().focused {
                continue;
            }

            // First check if global warning popup is showing and handle its dismissal
            if let FlowrsEvent::Key(key) = &event {
                let mut app = app.lock().unwrap();
                if app.warning_popup.is_some() {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.warning_popup = None;
                        }
                        _ => (),
                    }
                    // Consume the event when warning popup is showing
                    continue;
                }
            }

            // Then handle panel specific events, and send messages to the event channel
            let (fall_through_event, messages) = {
                let mut app = app.lock().unwrap();
                let ctx = app.nav_context.clone();
                match app.active_panel {
                    Panel::Config => app.configs.update(&event, &ctx),
                    Panel::Dag => app.dags.update(&event, &ctx),
                    Panel::DAGRun => app.dagruns.update(&event, &ctx),
                    Panel::TaskInstance => app.task_instances.update(&event, &ctx),
                    Panel::Logs => app.logs.update(&event, &ctx),
                }
            };

            // Set context IDs on target panels before sending messages to the worker.
            // Data sync from environment_state happens via sync_panel_data() on
            // navigation (Enter/Esc) and after worker completes API calls.
            {
                let mut app = app.lock().unwrap();
                for message in &messages {
                    app.set_context_from_message(message);
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
                if app.active_panel == Panel::Config {
                    app.ticks = 0;
                }
            }

            // then handle generic events
            let mut app = app.lock().unwrap();
            if fall_through_event == Some(FlowrsEvent::Tick) {
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
                        let panel = app.active_panel.clone();
                        app.sync_panel(&panel);
                    }
                    KeyCode::Esc | KeyCode::Left | KeyCode::Char('h') => {
                        app.previous_panel();
                        let panel = app.active_panel.clone();
                        app.sync_panel(&panel);
                    }
                    _ => {}
                }
            }
        }
    }
}
