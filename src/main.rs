use std::{error::Error, path::PathBuf};

use clap::Parser;

mod app;
mod commands;
mod model;
mod view;

use commands::{config::ConfigCommand, run::RunCommand};
use dirs::home_dir;

lazy_static::lazy_static! {
    pub static ref CONFIG_FILE: PathBuf = home_dir().unwrap().join(".flowrs");
}

#[derive(Parser)]
#[clap(name = "flowrs", author, version, about)]
enum FlowrsApp {
    Run(RunCommand),
    #[clap(subcommand)]
    Config(ConfigCommand),
}

impl FlowrsApp {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        match self {
            FlowrsApp::Run(cmd) => cmd.run(),
            FlowrsApp::Config(cmd) => cmd.run(),
        }
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let app = FlowrsApp::parse();
    app.run()
}
