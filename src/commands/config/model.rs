use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{Context, Result};
use clap::{Args, Parser};
use flowrs_config::{FlowrsConfig, ManagedService, Theme};
use inquire::validator::Validation;
use strum::Display;
use strum::EnumIter;
use url::Url;

#[derive(Parser, Debug)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct ConfigArgs {
    #[clap(subcommand)]
    pub command: Option<ConfigCommand>,

    #[clap(flatten)]
    pub global: GlobalSettings,

    #[clap(short, long)]
    pub file: Option<String>,
}

impl ConfigArgs {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            Some(cmd) => cmd.run().await,
            None => self.run_global_settings(),
        }
    }

    fn run_global_settings(&self) -> Result<()> {
        let path = self.file.as_ref().map(PathBuf::from);
        let mut config = FlowrsConfig::from_file(path.as_ref(), &crate::CONFIG_PATHS)?;

        if self.global.apply(&mut config) {
            config.write_to_file(&crate::CONFIG_PATHS)?;
        }

        println!("poll_interval_ms = {}", config.poll_interval_ms);
        println!("theme = {}", config.theme);
        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct GlobalSettings {
    /// API poll interval in milliseconds (minimum 500)
    #[clap(long)]
    pub poll_interval_ms: Option<PollIntervalMs>,

    /// Theme (auto, dark, light, catppuccin-latte, catppuccin-frappe, catppuccin-macchiato, catppuccin-mocha)
    #[clap(long)]
    pub theme: Option<Theme>,
}

impl GlobalSettings {
    /// Applies any set flags to `config`. Returns `true` if anything changed.
    pub fn apply(&self, config: &mut FlowrsConfig) -> bool {
        let mut changed = false;

        if let Some(v) = self.poll_interval_ms {
            let new_poll = v.into();
            if config.poll_interval_ms != new_poll {
                config.poll_interval_ms = new_poll;
                changed = true;
            }
        }

        if let Some(theme) = self.theme {
            if config.theme != theme {
                config.theme = theme;
                changed = true;
            }
        }

        changed
    }
}

/// Validated poll interval — rejects values below 500 at parse time.
#[derive(Debug, Clone, Copy)]
pub struct PollIntervalMs(u64);

impl From<PollIntervalMs> for u64 {
    fn from(v: PollIntervalMs) -> Self {
        v.0
    }
}

impl FromStr for PollIntervalMs {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: u64 = s
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid number: '{s}'"))?;
        if value < 500 {
            anyhow::bail!("must be at least 500 (got {value})");
        }
        Ok(Self(value))
    }
}

impl fmt::Display for PollIntervalMs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
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
    #[clap(long)]
    pub insecure: bool,
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
    #[clap(long)]
    pub insecure: bool,
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

/// Run a token command via `sh -c` and return its trimmed standard output.
///
/// Only the command string is persisted; running it here validates that it
/// actually yields a token, so a broken command is rejected at config time
/// rather than saved and left to fail on every later API call.
pub fn run_token_command(cmd: &str) -> Result<String> {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .with_context(|| format!("failed to run token command: {cmd}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("token command exited with {}: {}", output.status, stderr.trim());
    }

    let token = String::from_utf8(output.stdout)
        .context("token command produced non-UTF-8 output")?
        .trim()
        .to_string();

    if token.is_empty() {
        anyhow::bail!("token command produced no output: {cmd}");
    }

    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_command_returns_trimmed_output() {
        assert_eq!(run_token_command("echo my-token").unwrap(), "my-token");
    }

    #[test]
    fn token_command_errors_on_non_zero_exit() {
        assert!(run_token_command("exit 1").is_err());
    }

    #[test]
    fn token_command_errors_on_empty_output() {
        assert!(run_token_command("true").is_err());
    }
}
