use std::panic::PanicHookInfo;
use std::{
    fs::File,
    io::{self},
    path::Path,
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
use crate::app::{error::Result, state::App};

#[derive(Parser, Debug)]
pub struct RunCommand {
    #[clap(short, long)]
    pub file: Option<String>,
}

impl RunCommand {
    pub async fn run(&self) -> Result<()> {
        // setup logging
        setup_logging(Some("debug"))?;

        // setup panic hook
        std::panic::set_hook(Box::new(move |panic| {
            panic_hook(panic);
        }));

        // setup terminal
        let mut terminal = ratatui::init();

        // create app and run it
        let config = FlowrsConfig::from_file(self.file.as_deref().map(Path::new))?;
        let mut app = App::new(config).await?;

        run_app(&mut terminal, &mut app).await?;

        info!("Shutting down the terminal...");
        ratatui::restore();
        Ok(())
    }
}

fn setup_logging(debug: Option<&str>) -> Result<()> {
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
