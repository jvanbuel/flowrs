use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use clap::Parser;
use log::{info, LevelFilter};
use simplelog::{Config, WriteLogger};

use crate::airflow::config::FlowrsConfig;
use crate::app::run_app;
use crate::app::state::App;
use crate::CONFIG_PATHS;
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

        // Read config file
        let path = self.file.as_ref().map(PathBuf::from);
        let (config, errors) = FlowrsConfig::from_file(path.as_ref())?
            .expand_managed_services()
            .await?;

        // Generate warnings for legacy config conflict (only when no explicit --file)
        let mut warnings = Vec::new();
        if self.file.is_none() && CONFIG_PATHS.has_legacy_conflict {
            let legacy_path = dirs::home_dir()
                .map(|h| h.join(".flowrs"))
                .unwrap_or_else(|| PathBuf::from("~/.flowrs"));
            warnings.push(format!(
                "Configuration file found in both locations:\n  \
                 - {} (active)\n  \
                 - {} (ignored)\n\n\
                 Consider removing the legacy file.",
                CONFIG_PATHS.write_path.display(),
                legacy_path.display()
            ));
        }

        // setup terminal (includes panic hooks) and run app
        let mut terminal = ratatui::init();
        let app = App::new_with_errors_and_warnings(config, errors, warnings);
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

    WriteLogger::init(log_level, Config::default(), File::create(log_file)?)?;
    Ok(())
}
