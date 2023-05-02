use std::{error::Error, path::PathBuf};

use ascii_flowrs::ASCII_FLOWRS;
use clap::Parser;

mod app;
mod ascii_flowrs;
mod commands;
mod model;
mod ui;

use commands::{config::ConfigCommand, run::RunCommand};
use dirs::home_dir;

lazy_static::lazy_static! {
    pub static ref CONFIG_FILE: PathBuf = home_dir().unwrap().join(".flowrs");
}

#[derive(Parser)]
#[clap(name = "flowrs", version, about, before_help=ASCII_FLOWRS)]
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
    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        match &self.command {
            Some(FlowrsCommand::Run(cmd)) => cmd.run().await,
            Some(FlowrsCommand::Config(cmd)) => cmd.run(),
            None => RunCommand { file: None }.run().await,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = FlowrsApp::parse();
    app.run().await?;
    Ok(())
}
