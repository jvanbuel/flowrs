use std::panic::PanicHookInfo;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::{
    fs::File,
    io::{self},
};

use clap::Parser;
use crossterm::{
    event::DisableMouseCapture,
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use log::{info, LevelFilter};
use simplelog::{Config, WriteLogger};

use crate::airflow::config::FlowrsConfig;
use crate::app::run_app;
use crate::app::state::App;
use anyhow::Result;

#[derive(Parser, Debug)]
pub struct RunCommand {
    #[clap(short, long)]
    pub file: Option<String>,
}

impl RunCommand {
    pub async fn run(&self) -> Result<()> {
        // setup logging
        if let Ok(log_level) = std::env::var("FLOWRS_LOG") {
            setup_logging(&log_level)?;
        }

        // setup panic hook
        std::panic::set_hook(Box::new(move |panic| {
            panic_hook(panic);
        }));

        // setup terminal
        let mut terminal = ratatui::init();

        // create app and run it
        let path = self.file.as_ref().map(PathBuf::from);
        let config = FlowrsConfig::from_file(&path)?;
        let app = App::new(config)?;

        run_app(&mut terminal, Arc::new(Mutex::new(app))).await?;

        info!("Shutting down the terminal...");
        ratatui::restore();
        Ok(())
    }
}

fn setup_logging(log_level: &str) -> Result<()> {
    let log_file = format!(
        "./flowrs-debug-{}.log",
        chrono::Local::now().format("%Y%m%d%H%M%S")
    );
    let log_level = match log_level.to_lowercase().as_str() {
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info,
    };

    WriteLogger::init(
        log_level,
        Config::default(),
        File::create(log_file).unwrap(),
    )?;
    Ok(())
}

// #[cfg(debug_assertions)]
fn panic_hook(info: &PanicHookInfo<'_>) {
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

fn get_panic_info(info: &PanicHookInfo<'_>) -> (String, String) {
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
