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
mod rt;
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

// With the `dial9` feature, the runtime is built by dial9 and records a trace
// when `DIAL9_ENABLED=true` (see `docs/profiling.md`).
#[cfg_attr(feature = "dial9", dial9::main(config = dial9::recorder_from_env))]
#[cfg_attr(not(feature = "dial9"), tokio::main)]
async fn main() -> Result<()> {
    let app = FlowrsApp::parse();
    app.run().await?;
    // dial9 flushes its trace after `main` returns, which `exit` would skip.
    #[cfg(feature = "dial9")]
    return Ok(());
    #[cfg(not(feature = "dial9"))]
    std::process::exit(0);
}
