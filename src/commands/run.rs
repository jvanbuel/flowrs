use std::{io, path::Path, sync::Arc, thread};

use tokio::sync::Mutex;

use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use crate::app::{
    error::Result,
    filter::Filter,
    state::{App, Panel},
};
use crate::ui::ui;

#[derive(Parser, Debug)]
pub struct RunCommand {
    #[clap(short, long)]
    pub file: Option<String>,
}

impl RunCommand {
    pub async fn run(&self) -> Result<()> {
        // setup panic hook
        let original_hook = std::panic::take_hook();

        std::panic::set_hook(Box::new(move |panic| {
            reset_terminal().unwrap();
            original_hook(panic);
        }));

        // setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // create app and run it
        let path = self.file.as_ref().map(Path::new);
        let config = crate::app::auth::get_config(path)?;
        let app = Arc::new(Mutex::new(App::new(config).await));

        let res = run_app(&mut terminal, app).await;

        // restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        if let Err(err) = res {
            println!("{:?}", err)
        }

        Ok(())
    }
}

/// Resets the terminal.
fn reset_terminal() -> Result<()> {
    disable_raw_mode()?;
    crossterm::execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: Arc<Mutex<App>>) -> io::Result<()> {
    let app_nw = app.clone();

    tokio::spawn(async move {
        loop {
            let mut app = app_nw.lock().await;
            match app.active_panel {
                Panel::Dag => {
                    app.update_dags().await;
                    app.filter_dags();
                    // app.update_all_dagruns().await;
                }
                Panel::DAGRun => app.update_dagruns().await,
                Panel::TaskInstance => app.update_task_instances().await,
                _ => {}
            }

            drop(app);
            let ten_millis = std::time::Duration::from_millis(200);
            thread::sleep(ten_millis);
        }
    });

    loop {
        let mut app = app.lock().await;
        terminal.draw(|f| {
            ui(f, &mut app);
        })?;

        if let Event::Key(key) = event::read()? {
            if key.modifiers == KeyModifiers::CONTROL {
                match key.code {
                    KeyCode::Char('c') => return Ok(()),
                    KeyCode::Char('d') => return Ok(()),
                    _ => {}
                }
            }

            if app.filter.is_enabled() {
                mutate_filter(&mut app.filter, key.code);
                app.filter_dags();
            } else if app.active_popup {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => app.active_popup = false,
                    KeyCode::Enter => app.active_popup = false,
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('/') => app.toggle_search(),
                    KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => app.next_panel(),
                    KeyCode::Esc | KeyCode::Left | KeyCode::Char('h') => app.previous_panel(),
                    code => match app.active_panel {
                        Panel::Config => handle_key_code_config(code, &mut app).await,
                        Panel::Dag => handle_key_code_dag(code, &mut app).await,
                        Panel::DAGRun => handle_key_code_dagrun(code, &mut app).await,
                        Panel::TaskInstance => handle_key_code_task(code, &mut app).await,
                    },
                }
            }
        }
    }
}

async fn handle_key_code_config(code: KeyCode, app: &mut App) {
    match code {
        KeyCode::Down | KeyCode::Char('j') => app.configs.next(),
        KeyCode::Up | KeyCode::Char('k') => app.configs.previous(),
        _ => {}
    }
}

async fn handle_key_code_dag(code: KeyCode, app: &mut App) {
    match code {
        KeyCode::Down | KeyCode::Char('j') => app.filtered_dags.next(),
        KeyCode::Up | KeyCode::Char('k') => app.filtered_dags.previous(),
        KeyCode::Char('t') => app.toggle_current_dag().await,
        _ => {}
    }
}

async fn handle_key_code_dagrun(code: KeyCode, app: &mut App) {
    match code {
        KeyCode::Down | KeyCode::Char('j') => app.dagruns.next(),
        KeyCode::Up | KeyCode::Char('k') => app.dagruns.previous(),
        KeyCode::Char('c') => app.clear_dagrun().await,
        _ => {}
    }
}

async fn handle_key_code_task(code: KeyCode, app: &mut App) {
    match code {
        KeyCode::Down | KeyCode::Char('j') => app.taskinstances.next(),
        KeyCode::Up | KeyCode::Char('k') => app.taskinstances.previous(),
        _ => {}
    }
}

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
