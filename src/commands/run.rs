use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use clap::Parser;
use crossterm::event::{DisableFocusChange, EnableFocusChange};
use crossterm::ExecutableCommand;
use log::{info, LevelFilter, Log, Metadata, Record};

use crate::airflow::config::FlowrsConfig;
use crate::app::run_app;
use crate::app::state::App;
use crate::CONFIG_PATHS;
use anyhow::Result;

struct FileLogger {
    file: Mutex<File>,
    level: LevelFilter,
}

impl Log for FileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            if let Ok(mut file) = self.file.lock() {
                let _ = writeln!(
                    file,
                    "[{}] {} - {}",
                    record.level(),
                    record.target(),
                    record.args()
                );
            }
        }
    }

    fn flush(&self) {
        if let Ok(mut file) = self.file.lock() {
            let _ = file.flush();
        }
    }
}

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
            let legacy_path =
                dirs::home_dir().map_or_else(|| PathBuf::from("~/.flowrs"), |h| h.join(".flowrs"));
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
        std::io::stdout().execute(EnableFocusChange)?;

        let app = App::new_with_errors_and_warnings(config, errors, warnings);
        let result = run_app(&mut terminal, Arc::new(Mutex::new(app))).await;

        info!("Shutting down the terminal...");
        std::io::stdout().execute(DisableFocusChange)?;
        ratatui::restore();
        result
    }
}

fn setup_logging(log_level: &str) -> Result<()> {
    let log_dir = dirs::state_dir().map_or_else(|| PathBuf::from("."), |p| p.join("flowrs"));
    std::fs::create_dir_all(&log_dir)?;

    let log_file = format!(
        "{}/flowrs-debug-{}.log",
        log_dir.display(),
        chrono::Local::now().format("%Y%m%d%H%M%S")
    );
    let level = match log_level.to_lowercase().as_str() {
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info,
    };

    let logger = FileLogger {
        file: Mutex::new(File::create(log_file)?),
        level,
    };
    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(level);
    Ok(())
}
