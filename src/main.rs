use std::sync::LazyLock;

use clap::Parser;
use ui::constants::ASCII_LOGO;

// mimalloc handles this application's many short-lived allocations (per-poll
// clones, log/format strings, render buffers) more efficiently than the system
// allocator (M-MIMALLOC-APPS).
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod airflow;
mod app;
mod commands;
mod ui;

use anyhow::Result;
use commands::config::model::ConfigArgs;
use commands::run::RunCommand;
use flowrs_config::paths::ConfigPaths;

pub static CONFIG_PATHS: LazyLock<ConfigPaths> = LazyLock::new(ConfigPaths::resolve);

#[derive(Parser)]
#[clap(name="flowrs", bin_name="flowrs", version, about, before_help=ASCII_LOGO)]
struct FlowrsApp {
    #[clap(subcommand)]
    command: Option<FlowrsCommand>,
}

#[derive(Parser)]
enum FlowrsCommand {
    Run(RunCommand),
    Config(ConfigArgs),
}

impl FlowrsApp {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            Some(FlowrsCommand::Run(cmd)) => cmd.run().await,
            Some(FlowrsCommand::Config(cmd)) => cmd.run().await,
            None => RunCommand { file: None }.run().await,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = FlowrsApp::parse();
    app.run().await?;
    std::process::exit(0);
}
