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

pub mod events;
pub mod model;
pub mod state;
pub mod worker;

/// Runs the terminal-based UI main loop and spawns a background worker for the application.
///
/// The function draws the UI, processes user and internal events, dispatches panel-specific
/// updates to the shared application state, sends messages to the worker, and handles global
/// keybindings (including quitting and writing config). It also creates an Airflow client
/// from the currently active server (if any) and passes it to the spawned worker.
///
/// # Parameters
///
/// - `terminal`: terminal instance used for rendering the UI.
/// - `app`: shared application state wrapped in `Arc<Mutex<_>>`.
///
/// # Returns
///
/// `Ok(())` when the UI loop exits normally; an error if a terminal draw or configuration write
/// operation fails.
///
/// # Examples
///
/// ```no_run
/// use std::sync::{Arc, Mutex};
/// use tokio;
/// use ratatui::Terminal;
/// // Assume `MyBackend` implements `ratatui::backend::Backend` and `App::new()` exists.
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let backend = MyBackend::new();
///     let mut terminal = Terminal::new(backend)?;
///     let app = Arc::new(Mutex::new(App::new()));
///     run_app(&mut terminal, app).await
/// }
/// ```
pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: Arc<Mutex<App>>) -> Result<()> {
    let mut events = EventGenerator::new(200);
    let ui_app = app.clone();
    let worker_app = app.clone();

    let (tx_worker, rx_worker) = tokio::sync::mpsc::channel::<WorkerMessage>(100);

    log::info!("Starting worker");
    let airflow_client;
    {
        let app = app.lock().unwrap();
        let previously_active_server = &app.config.active_server;
        let airflow_config = match previously_active_server {
            Some(server) => app
                .config
                .servers
                .as_ref()
                .and_then(|servers| servers.iter().find(|s| s.name == *server)),
            _ => None,
        };
        airflow_client = airflow_config.and_then(|config| create_client(config.clone()).ok());
    }

    log::info!("Spawning worker");
    tokio::spawn(async move {
        Worker::new(worker_app, airflow_client, rx_worker)
            .run()
            .await
    });

    loop {
        terminal.draw(|f| {
            debug!("Drawing UI");
            draw_ui(f, &ui_app);
        })?;

        if let Some(event) = events.next().await {
            // If the app is loading, only tick events are allowed
            if app.lock().unwrap().loading && event != FlowrsEvent::Tick {
                continue;
            }
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

            for message in messages {
                tx_worker.send(message).await.unwrap();
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
            }
            if let FlowrsEvent::Key(key) = event {
                // Handle exit key events
                if key.modifiers == KeyModifiers::CONTROL {
                    match key.code {
                        KeyCode::Char('c') => return Ok(()),
                        KeyCode::Char('d') => return Ok(()),
                        _ => {}
                    }
                }
                // Handle other key events
                match key.code {
                    KeyCode::Char('q') => {
                        app.config.write_to_file()?;
                        return Ok(());
                    }
                    KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => app.next_panel(),
                    KeyCode::Esc | KeyCode::Left | KeyCode::Char('h') => app.previous_panel(),
                    _ => {}
                }
            }
        }
    }
}