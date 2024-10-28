use std::{
    io,
    sync::{Arc, Mutex},
};

use crossterm::event::{KeyCode, KeyModifiers};
use events::{custom::FlowrsEvent, generator::EventGenerator};
use log::debug;
use model::Model;
use ratatui::{prelude::Backend, Terminal};
use state::{App, Panel};
use worker::{Worker, WorkerMessage};

use crate::{airflow::client::AirFlowClient, ui::draw_ui};

pub mod error;
pub mod events;
pub mod model;
pub mod state;
pub mod worker;

pub async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: Arc<Mutex<App>>,
) -> io::Result<()> {
    let mut events = EventGenerator::new(200);
    let ui_app = app.clone();
    let worker_app = app.clone();

    let (tx_worker, rx_worker) = tokio::sync::mpsc::channel::<WorkerMessage>(100);

    log::info!("Starting worker");
    let airflow_client: AirFlowClient;
    {
        let app = app.lock().unwrap();
        airflow_client = AirFlowClient::from(app.configs.all[0].clone());
    }

    log::info!("Spawning worker");
    tokio::spawn(async move {
        Worker::new(worker_app, airflow_client, rx_worker)
            .run()
            .await
    });

    log::info!("Registering worker");
    app.lock().unwrap().register_worker(tx_worker);

    loop {
        terminal.draw(|f| {
            debug!("Drawing UI");
            draw_ui(f, &ui_app);
        })?;

        if let Some(event) = events.next().await {
            // First handle panel specific events, and send messages to the event channel
            let mut app = app.lock().unwrap();
            let fall_through_event = match app.active_panel {
                Panel::Config => app.configs.update(&event).await,
                Panel::Dag => app.dags.update(&event).await,
                Panel::DAGRun => app.dagruns.update(&event).await,
                Panel::TaskInstance => {
                    unimplemented!()
                }
            };
            if fall_through_event.is_none() {
                continue;
            }

            // then handle generic events
            match event {
                FlowrsEvent::Tick => {
                    debug!("Tick event");
                    app.ticks += 1;
                }
                FlowrsEvent::Key(key) => {
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
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('?') => unimplemented!(),
                        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => app.next_panel(),
                        KeyCode::Esc | KeyCode::Left | KeyCode::Char('h') => app.previous_panel(),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}
