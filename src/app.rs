use std::sync::{Arc, Mutex};

use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use events::{custom::FlowrsEvent, generator::EventGenerator};
use log::debug;
use ratatui::{prelude::Backend, Terminal};
use state::App;
use worker::{Dispatcher, Request};

use crate::airflow::client::FlowrsClient;
use crate::airflow::model::common::EnvironmentKey;

use crate::{ui::draw_ui, CONFIG_PATHS};

pub mod events;
pub mod model;
pub mod navigation;
pub mod state;
pub mod worker;

pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: Arc<Mutex<App>>) -> Result<()>
where
    <B as Backend>::Error: 'static + std::marker::Send + std::marker::Sync,
{
    let mut events = EventGenerator::new(200);
    let ui_app = Arc::clone(&app);
    let worker_app = Arc::clone(&app);

    let (tx_worker, rx_worker) = tokio::sync::mpsc::channel::<Request>(100);

    log::info!("Initializing environment state");
    {
        let mut app = app.lock().unwrap();

        // Clone servers to avoid borrow checker issues
        let servers = app.config.servers.clone();
        let active_server_name = app.config.active_server.clone();

        // Initialize all environments with their clients
        for server_config in servers {
            if let Ok(client) = FlowrsClient::new(&server_config) {
                let env_data = state::environment_state::EnvironmentData::new(Arc::new(client));
                app.environment_state
                    .environments
                    .insert(EnvironmentKey::from(server_config.name.clone()), env_data);
            } else {
                log::error!(
                    "Failed to create client for server '{}'; skipping",
                    server_config.name
                );
            }
        }

        // Set the active environment if one was configured
        if let Some(active_server_name) = active_server_name {
            app.environment_state
                .set_active_environment(EnvironmentKey::from(active_server_name));
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

            // Then handle panel specific events. A navigation key the panel
            // did not consume moves forward or back.
            let (fall_through_event, messages, environment) = {
                let mut app = app.lock().unwrap();
                let (fall_through_event, mut messages) = app.update_active_panel(&event);
                if let Some(FlowrsEvent::Key(key)) = &fall_through_event {
                    match key.code {
                        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                            messages.extend(app.go_forward());
                        }
                        KeyCode::Esc | KeyCode::Left | KeyCode::Char('h') => app.go_back(),
                        _ => {}
                    }
                }
                // Bind the messages to the environment they were made for;
                // after an environment switch that is the new one.
                let environment = app
                    .environment_state
                    .active_environment
                    .as_ref()
                    .map(ToString::to_string);
                (fall_through_event, messages, environment)
            };

            // Send messages to the worker for async processing
            for message in messages {
                let request = Request {
                    environment: environment.clone(),
                    message,
                };
                if let Err(e) = tx_worker.send(request).await {
                    log::error!("Failed to send message to worker: {e}");
                }
            }
            if fall_through_event.is_none() {
                continue;
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
                if key.code == KeyCode::Char('q') {
                    // Persist config (e.g. active-server selection) on exit, but
                    // a save failure should not turn a normal quit into an error
                    // exit.
                    if let Err(e) = app.config.write_to_file(&CONFIG_PATHS) {
                        log::error!("Failed to save config on exit: {e}");
                    }
                    return Ok(());
                }
            }
        }
    }
}
