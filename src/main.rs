use std::path::PathBuf;

use clap::Parser;
use ui::constants::ASCII_LOGO;

mod airflow;
mod app;
mod commands;
mod ui;

use commands::config::model::ConfigCommand;
use commands::run::RunCommand;
use dirs::home_dir;

lazy_static::lazy_static! {
    pub static ref CONFIG_FILE: PathBuf = home_dir().unwrap().join(".flowrs");
}

use app::error::Result;

#[derive(Parser)]
#[clap(name="flowrs", bin_name="flowrs", version, about, before_help=ASCII_LOGO)]
struct FlowrsApp {
    #[clap(subcommand)]
    command: Option<FlowrsCommand>,
}

#[derive(Parser)]
enum FlowrsCommand {
    Run(RunCommand),
    #[clap(subcommand)]
    Config(ConfigCommand),
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
