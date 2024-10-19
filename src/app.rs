use std::{io, sync::atomic::Ordering::Relaxed};

use crossterm::event::{KeyCode, KeyModifiers};
use events::{custom::FlowrsEvent, generator::EventGenerator};
use log::debug;
use model::{filter::Filter, Model};
use ratatui::{prelude::Backend, Terminal};
use state::{App, Panel};

use crate::ui::draw_ui;

pub mod error;
pub mod events;
pub mod model;
pub mod state;

pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    let events = EventGenerator::new(50);

    loop {
        terminal.draw(|f| {
            debug!("Drawing UI");
            draw_ui(f, app);
        })?;

        if let Ok(event) = events.next() {
            // first handle generic events which don't require API calls, so it's OK to lock the app
            {
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
                            KeyCode::Char('?') => app.active_panel = Panel::Help,
                            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                                app.next_panel()
                            }
                            KeyCode::Esc | KeyCode::Left | KeyCode::Char('h') => {
                                app.previous_panel()
                            }
                            _ => {}
                        }
                    }
                    FlowrsEvent::Mouse(_) => {}
                }
            }

            // then handle panel specific events
            match app.active_panel {
                Panel::Config => app.configs.update(event).await,
                Panel::Dag => app.dags.update(event).await,
                Panel::DAGRun => unimplemented!(),
                Panel::TaskInstance => {
                    unimplemented!()
                }
                _ => {}
            }
        }
    }
}

// async fn handle_key_code_config(code: KeyCode, app: &mut App) {
//     match code {
//         KeyCode::Down | KeyCode::Char('j') => {
//             app.configs.filtered.next();
//             let selected_config = app.configs.state.selected().unwrap_or_default();
//             let new_config = &app.configs.items[selected_config];
//             info!("Selected config: {:?}", new_config);
//             let new_context = Arc::new(FlowrsContext::new(
//                 AirFlowClient::new(new_config.clone()).unwrap(),
//             ));
//             app.context = new_context.clone();
//             app.dags.context = new_context;
//         }
//         KeyCode::Up | KeyCode::Char('k') => {
//             app.configs.previous();
//             let selected_config = app.configs.state.selected().unwrap_or_default();
//             let new_config = &app.configs.items[selected_config];

//             let new_context = Arc::new(FlowrsContext::new(
//                 AirFlowClient::new(new_config.clone()).unwrap(),
//             ));
//             app.context = new_context;
//         }
//         _ => {}
//     }
// }

fn mutate_filter(filter: &mut Filter, code: KeyCode) {
    match code {
        KeyCode::Esc | KeyCode::Enter => {
            filter.toggle();
        }
        KeyCode::Backspace => {
            if let Some(ref mut prefix) = filter.prefix {
                prefix.pop();
            }
        }
        KeyCode::Char(c) => match filter.prefix {
            Some(ref mut prefix) => {
                prefix.push(c);
            }
            None => {
                filter.prefix = Some(c.to_string());
            }
        },
        _ => {}
    }
}
