use crate::airflow::config::ManagedService;
use anyhow::Result;
use clap::Parser;
use inquire::validator::Validation;
use strum::Display;
use strum::EnumIter;
use url::Url;

#[derive(Parser, Debug)]
pub enum ConfigCommand {
    Add(AddCommand),
    #[clap(alias = "rm")]
    Remove(RemoveCommand),
    Update(UpdateCommand),
    #[clap(alias = "ls")]
    List(ListCommand),
    Enable(ManagedServiceCommand),
    Disable(ManagedServiceCommand),
}

impl ConfigCommand {
    pub fn run(&self) -> Result<()> {
        match self {
            ConfigCommand::Add(cmd) => cmd.run(),
            ConfigCommand::Remove(cmd) => cmd.run(),
            ConfigCommand::Update(cmd) => cmd.run(),
            ConfigCommand::List(cmd) => cmd.run(),
            ConfigCommand::Enable(cmd) => cmd.run(),
            ConfigCommand::Disable(cmd) => cmd.disable(),
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
pub struct ListCommand {
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

#[derive(Parser, Debug)]
pub struct ManagedServiceCommand {
    #[clap(short, long)]
    pub managed_service: Option<ManagedService>,
    #[clap(short, long)]
    pub file: Option<String>,
}

type Command = Option<String>;

pub fn validate_endpoint(
    endpoint: &str,
) -> std::result::Result<Validation, Box<dyn std::error::Error + Send + Sync>> {
    match Url::parse(endpoint) {
        Ok(_) => Ok(Validation::Valid),
        Err(error) => Ok(Validation::Invalid(error.into())),
    }
}
