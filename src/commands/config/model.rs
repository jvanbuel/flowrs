use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{Context, Result};
use clap::{Args, Parser};
use flowrs_config::{AirflowAuth, CookieAuth, FlowrsConfig, ManagedService, Theme};
use inquire::validator::Validation;
use inquire::{Select, Text};
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
    Cookie,
}

#[derive(Parser, Debug)]
pub struct ManagedServiceCommand {
    #[clap(short, long)]
    pub managed_service: Option<ManagedService>,
    #[clap(short, long)]
    pub file: Option<String>,
}

type Command = Option<String>;

/// Prompt for cookie authentication: either a pasted value or a helper command
/// whose stdout is the `Cookie` header. Shared by `config add` and `config update`.
pub fn prompt_cookie_auth() -> Result<AirflowAuth> {
    let source = Select::new(
        "cookie source",
        vec![
            "paste a cookie value",
            "run a command that prints the cookie",
        ],
    )
    .with_help_message("A pasted cookie is static; a command lets flowrs refresh a rotating cookie")
    .prompt()?;

    if source == "run a command that prints the cookie" {
        let cmd = Text::new("cmd")
            .with_help_message("Command whose stdout is the Cookie header value")
            .prompt()?;
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .with_context(|| format!("Failed to execute cookie command: {cmd}"))?;
        let cookie = String::from_utf8(output.stdout)?.trim().to_string();
        anyhow::ensure!(
            output.status.success() && !cookie.is_empty(),
            "cookie command exited with {:?} or printed nothing",
            output.status.code()
        );
        Ok(AirflowAuth::Cookie(CookieAuth::Command { cmd }))
    } else {
        let cookie = inquire::Password::new("cookie")
            .with_display_toggle_enabled()
            .with_help_message(
                "Paste the Cookie header from your browser session (e.g. session=abc123)",
            )
            .prompt()?;
        Ok(AirflowAuth::Cookie(CookieAuth::Static { cookie }))
    }
}

#[allow(
    clippy::unnecessary_wraps,
    reason = "signature kept uniform with the inquire validator interface"
)]
pub fn validate_endpoint(
    endpoint: &str,
) -> Result<Validation, Box<dyn std::error::Error + Send + Sync>> {
    match Url::parse(endpoint) {
        Ok(_) => Ok(Validation::Valid),
        Err(error) => Ok(Validation::Invalid(error.into())),
    }
}
