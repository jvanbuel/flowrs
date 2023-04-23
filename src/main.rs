use std::error::Error;

use clap::Parser;

mod app;
mod commands;
mod model;
mod view;

use commands::{config::ConfigCommand, run::RunCommand};

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
