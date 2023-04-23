use std::error::Error;

use clap::Parser;

#[derive(Parser, Debug)]
pub enum ConfigCommand {
    Add(AddCommand),
}

#[derive(Parser, Debug)]
pub struct AddCommand {}

impl AddCommand {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        println!("This is a test");
        Ok(())
    }
}

impl ConfigCommand {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        match self {
            ConfigCommand::Add(cmd) => cmd.run(),
        }
    }
}
