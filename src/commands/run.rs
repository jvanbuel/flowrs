use std::{
    fs::File,
    io::{self, Stdout},
    path::Path,
    sync::Arc,
    thread,
};

use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::{info, LevelFilter};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use simplelog::{Config, WriteLogger};
use std::panic::PanicInfo;
use tokio::sync::Mutex;

use crate::app::{
    client::AirFlowClient,
    config::FlowrsConfig,
    error::Result,
    events::{custom::CustomEvent, generator::EventGenerator},
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
        // setup logging
        setup_logging(None)?;

        // setup panic hook
        std::panic::set_hook(Box::new(move |panic| {
            panic_hook(panic);
        }));

        // setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // create app and run it
        let path = self.file.as_ref().map(Path::new);
        let config = FlowrsConfig::from_file(path)?;
        let app = Arc::new(Mutex::new(App::new(config).await?));

        run_app(&mut terminal, app).await?;

        info!("Shutting down the terminal...");
        terminal.show_cursor()?;
        shutdown(terminal)
    }
}

fn shutdown(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn setup_logging(debug: Option<String>) -> Result<()> {
    let log_file = format!(
        "./flowrs-debug-{}.log",
        chrono::Local::now().format("%Y%m%d%H%M%S")
    );
    let log_level = debug
        .map(|level| match level.to_lowercase().as_str() {
            "debug" => LevelFilter::Debug,
            "trace" => LevelFilter::Trace,
            "warn" => LevelFilter::Warn,
            "error" => LevelFilter::Error,
            _ => LevelFilter::Info,
        })
        .unwrap_or_else(|| LevelFilter::Info);

    WriteLogger::init(
        log_level,
        Config::default(),
        File::create(log_file).unwrap(),
    )?;
    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: Arc<Mutex<App>>) -> io::Result<()> {
    let app_nw = app.clone();

    let event_generator = EventGenerator::new(50);

    tokio::spawn(async move {
        loop {
            // this is completely wrong! You don't need to lock your state while making a request...
            // The lock should be acquired only when you're updating the state, after the API request has returned.
            let active_panel: Panel;
            let client: AirFlowClient;
            {
                let app = app_nw.lock().await;
                active_panel = app.active_panel.clone();
                client = app.client.clone();
            }
            match active_panel {
                Panel::Dag => {
                    {
                        let mut app = app_nw.lock().await;
                        app.is_loading = true;
                    }
                    let dags = client.list_dags().await;
                    // filter dags
                    if let Ok(dag_list) = dags {
                        let mut app = app_nw.lock().await;
                        app.all_dags = dag_list;
                        app.filter_dags();
                        app.is_loading = false;
                    }
                }
                Panel::DAGRun => {
                    let dag_id: String;
                    {
                        let mut app = app_nw.lock().await;
                        dag_id = app.filtered_dags.items
                            [app.filtered_dags.state.selected().unwrap_or_default()]
                        .dag_id
                        .clone();
                        app.is_loading = true;
                    };
                    info!("Fetching dagruns for dag_id: {}", dag_id);
                    let dagruns = client.list_dagruns(&dag_id).await;
                    if let Ok(dagruns) = dagruns {
                        let mut app = app_nw.lock().await;
                        app.dagruns.items = dagruns.dag_runs;
                        app.is_loading = false;
                    }
                }
                Panel::TaskInstance => {
                    let dag_id = {
                        let app = app_nw.lock().await;
                        app.filtered_dags.items
                            [app.filtered_dags.state.selected().unwrap_or_default()]
                        .dag_id
                        .clone()
                    };
                    let dag_run_id = {
                        let app = app_nw.lock().await;
                        app.dagruns.items[app.dagruns.state.selected().unwrap_or_default()]
                            .dag_run_id
                            .clone()
                    };
                    info!(
                        "Fetching task instances for dag_id: {}, dag_run_id: {}",
                        dag_id, dag_run_id
                    );
                    let taskinstances = client.list_task_instances(&dag_id, &dag_run_id).await;
                    if let Ok(taskinstances) = taskinstances {
                        let mut app = app_nw.lock().await;
                        app.all_taskinstances = taskinstances;
                    }
                }
                _ => {}
            }

            let refresh_rate = std::time::Duration::from_millis(1000);
            thread::sleep(refresh_rate);
        }
    });

    loop {
        let mut app = app.lock().await;
        terminal.draw(|f| {
            ui(f, &mut app);
        })?;

        if let Ok(event) = event_generator.next() {
            match event {
                CustomEvent::Tick => continue,
                CustomEvent::Key(key) => {
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
                            KeyCode::Char('?') => app.active_panel = Panel::Help,
                            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                                app.next_panel()
                            }
                            KeyCode::Esc | KeyCode::Left | KeyCode::Char('h') => {
                                app.previous_panel()
                            }
                            code => match app.active_panel {
                                Panel::Config => handle_key_code_config(code, &mut app).await,
                                Panel::Dag => handle_key_code_dag(code, &mut app).await,
                                Panel::DAGRun => handle_key_code_dagrun(code, &mut app).await,
                                Panel::TaskInstance => handle_key_code_task(code, &mut app).await,
                                Panel::Help => {}
                            },
                        }
                    }
                }
                CustomEvent::Mouse(_) => {}
            }
        }
    }
}

async fn handle_key_code_config(code: KeyCode, app: &mut App) {
    match code {
        KeyCode::Down | KeyCode::Char('j') => {
            app.configs.next();
            let selected_config = app.configs.state.selected().unwrap_or_default();
            let new_config = &app.configs.items[selected_config];
            info!("Selected config: {:?}", new_config);
            app.client = AirFlowClient::new(new_config.clone()).unwrap();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.configs.previous();
            let selected_config = app.configs.state.selected().unwrap_or_default();
            let new_config = &app.configs.items[selected_config];
            app.client = AirFlowClient::new(new_config.clone()).unwrap();
        }
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

// #[cfg(debug_assertions)]
fn panic_hook(info: &PanicInfo<'_>) {
    use backtrace::Backtrace;
    use crossterm::style::Print;

    let (msg, location) = get_panic_info(info);

    let stacktrace: String = format!("{:?}", Backtrace::new()).replace('\n', "\n\r");

    disable_raw_mode().unwrap();
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        Print(format!(
            "thread '<unnamed>' panicked at '{}', {}\n\r{}",
            msg, location, stacktrace
        )),
    )
    .unwrap();
}

fn get_panic_info(info: &PanicInfo<'_>) -> (String, String) {
    let location = info.location().unwrap();

    let msg = match info.payload().downcast_ref::<&'static str>() {
        Some(s) => *s,
        None => match info.payload().downcast_ref::<String>() {
            Some(s) => &s[..],
            None => "Box<Any>",
        },
    };

    (msg.to_string(), format!("{}", location))
}
