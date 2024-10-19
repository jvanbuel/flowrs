use std::{
    io,
    sync::{atomic::Ordering::Relaxed, Arc},
};

use crossterm::event::{KeyCode, KeyModifiers};
use events::{custom::FlowrsEvent, generator::EventGenerator};
use log::debug;
use model::Model;
use ratatui::{prelude::Backend, Terminal};
use state::{App, FlowrsContext, Panel};

use crate::{airflow::client::AirFlowClient, ui::draw_ui};

pub mod error;
pub mod events;
pub mod model;
pub mod state;

pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    let events = EventGenerator::new(200);

    loop {
        terminal.draw(|f| {
            debug!("Drawing UI");
            draw_ui(f, app);
        })?;

        if let Ok(event) = events.next() {
            // First handle panel specific events, and send messages to the event channel
            match app.active_panel {
                Panel::Config => {
                    if let Some(msg) = app.configs.update(&event).await {
                        debug!("Sending message to event channel: {:?}", msg);
                        events.tx_event.send(msg).unwrap();
                    }
                }
                Panel::Dag => _ = app.dags.update(&event).await,
                Panel::DAGRun => unimplemented!(),
                Panel::TaskInstance => {
                    unimplemented!()
                }
            }

            // then handle generic events
            match event {
                FlowrsEvent::Tick => {
                    debug!("Tick event");
                    app.context.ticks.fetch_add(1, Relaxed);
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
                FlowrsEvent::ConfigSelected(index) => {
                    let new_client: AirFlowClient =
                        app.configs.filtered.items[index].clone().into();
                    app.context = Arc::new(FlowrsContext::new(new_client));
                    app.update_contexts();
                }
                _ => {}
            }
        }
    }
}

// fn mutate_filter(filter: &mut Filter, code: KeyCode) {
//     match code {
//         KeyCode::Esc | KeyCode::Enter => {
//             filter.toggle();
//         }
//         KeyCode::Backspace => {
//             if let Some(ref mut prefix) = filter.prefix {
//                 prefix.pop();
//             }
//         }
//         KeyCode::Char(c) => match filter.prefix {
//             Some(ref mut prefix) => {
//                 prefix.push(c);
//             }
//             None => {
//                 filter.prefix = Some(c.to_string());
//             }
//         },
//         _ => {}
//     }
// }
