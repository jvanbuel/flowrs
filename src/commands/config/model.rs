use crate::app::error::Result;
use clap::Parser;
use strum::Display;
use strum::EnumIter;

#[derive(Parser, Debug)]
pub enum ConfigCommand {
    Add(AddCommand),
    #[clap(alias = "rm")]
    Remove(RemoveCommand),
    Update(UpdateCommand),
}

impl ConfigCommand {
    pub fn run(&self) -> Result<()> {
        match self {
            ConfigCommand::Add(cmd) => cmd.run(),
            ConfigCommand::Remove(cmd) => cmd.run(),
            ConfigCommand::Update(cmd) => cmd.run(),
        }
    }
}

#[derive(Parser, Debug)]
pub struct AddCommand {
    #[clap(short, long)]
    pub file: Option<String>,
}

#[derive(Parser, Debug)]
pub struct RemoveCommand {
    pub name: Option<String>,
    #[clap(short, long)]
    pub file: Option<String>,
}

#[derive(Parser, Debug)]
pub struct UpdateCommand {
    pub name: Option<String>,
    #[clap(short, long)]
    pub file: Option<String>,
}

#[derive(EnumIter, Debug, Display)]
pub enum ConfigOption {
    BasicAuth,
    Token(Command),
}

type Command = Option<String>;
