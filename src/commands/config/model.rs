use flowrs_config::ManagedService;
use anyhow::Result;
use clap::Parser;
use inquire::validator::Validation;
use strum::Display;
use strum::EnumIter;
use url::Url;

#[derive(Parser, Debug)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct ConfigArgs {
    #[clap(subcommand)]
    pub command: Option<ConfigCommand>,

    /// API poll interval in milliseconds (minimum 500)
    #[clap(long)]
    pub poll_interval_ms: Option<u64>,

    #[clap(short, long)]
    pub file: Option<String>,
}

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

impl ConfigArgs {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            Some(cmd) => cmd.run().await,
            None => self.run_global_settings(),
        }
    }

    fn run_global_settings(&self) -> Result<()> {
        use std::path::PathBuf;

        use flowrs_config::FlowrsConfig;

        let path = self.file.as_ref().map(PathBuf::from);
        let mut config = FlowrsConfig::from_file(path.as_ref(), &crate::CONFIG_PATHS)?;

        let has_changes = self.poll_interval_ms.is_some();

        if !has_changes {
            println!("poll_interval_ms = {}", config.poll_interval_ms);
            return Ok(());
        }

        if let Some(value) = self.poll_interval_ms {
            if value < 500 {
                anyhow::bail!("poll_interval_ms must be at least 500 (got {value})");
            }
            config.poll_interval_ms = value;
            config.write_to_file(&crate::CONFIG_PATHS)?;
            println!("poll_interval_ms set to {value}");
        }

        Ok(())
    }
}

impl ConfigCommand {
    pub async fn run(&self) -> Result<()> {
        match self {
            Self::Add(cmd) => cmd.run(),
            Self::Remove(cmd) => cmd.run(),
            Self::Update(cmd) => cmd.run(),
            Self::List(cmd) => cmd.run(),
            Self::Enable(cmd) => cmd.run().await,
            Self::Disable(cmd) => cmd.disable(),
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

#[allow(clippy::unnecessary_wraps)]
pub fn validate_endpoint(
    endpoint: &str,
) -> Result<Validation, Box<dyn std::error::Error + Send + Sync>> {
    match Url::parse(endpoint) {
        Ok(_) => Ok(Validation::Valid),
        Err(error) => Ok(Validation::Invalid(error.into())),
    }
}
