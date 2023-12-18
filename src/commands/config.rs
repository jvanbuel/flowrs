use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use log::info;

use clap::Parser;
use inquire::validator::Validation;
use inquire::Select;
use strum::Display;
use url::Url;

use crate::app::config::AirflowAuth;
use crate::app::config::AirflowConfig;
use crate::app::config::BasicAuth;
use crate::app::config::FlowrsConfig;
use crate::app::config::TokenCmd;
use crate::app::error::Result;
use crate::CONFIG_FILE;
use strum::{EnumIter, IntoEnumIterator};

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
    file: Option<String>,
}

#[derive(Parser, Debug)]
pub struct RemoveCommand {
    name: Option<String>,
    #[clap(short, long)]
    file: Option<String>,
}

#[derive(Parser, Debug)]
pub struct UpdateCommand {
    name: Option<String>,
    #[clap(short, long)]
    file: Option<String>,
}

#[derive(EnumIter, Debug, Display)]
pub enum ConfigOption {
    BasicAuth,
    Token(Command),
}

type Command = Option<String>;

impl AddCommand {
    pub fn run(&self) -> Result<()> {
        let name = inquire::Text::new("name").prompt()?;
        let endpoint = inquire::Text::new("endpoint")
            .with_validator(validate_endpoint)
            .prompt()?;

        let auth_type =
            Select::new("authentication type", ConfigOption::iter().collect()).prompt()?;

        let new_config = match auth_type {
            ConfigOption::BasicAuth => {
                let username = inquire::Text::new("username").prompt()?;
                let password = inquire::Password::new("password")
                    .with_display_toggle_enabled()
                    .prompt()?;

                AirflowConfig {
                    name,
                    endpoint,
                    auth: AirflowAuth::BasicAuth(BasicAuth { username, password }),
                }
            }
            ConfigOption::Token(_) => {
                let cmd = Some(inquire::Text::new("cmd").prompt()?);
                let token: String;
                if let Some(cmd) = &cmd {
                    info!("🔑 Running command: {}", cmd);
                    let output = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(cmd)
                        .output()
                        .expect("failed to execute process");
                    token = String::from_utf8(output.stdout)?;
                } else {
                    token = inquire::Text::new("token").prompt()?;
                }

                AirflowConfig {
                    name,
                    endpoint,
                    auth: AirflowAuth::TokenAuth(TokenCmd { cmd, token }),
                }
            }
        };

        let path = self.file.as_ref().map(Path::new);
        let mut config = FlowrsConfig::from_file(path)?;
        config
            .servers
            .retain(|server| server.name != new_config.name);

        config.servers.push(new_config);

        write_config(&config, path)?;

        println!("✅ Config added successfully!");
        Ok(())
    }
}

impl RemoveCommand {
    pub fn run(&self) -> Result<()> {
        let path = self.file.as_ref().map(Path::new);
        let mut config = FlowrsConfig::from_file(path)?;

        let name = match self.name {
            None => Select::new(
                "name",
                config
                    .servers
                    .iter()
                    .map(|server| server.name.clone())
                    .collect(),
            )
            .prompt()?,
            Some(ref name) => name.to_string(),
        };
        config.servers.retain(|server| server.name != name);

        write_config(&config, path)?;

        println!("✅ Config '{}' removed successfully!", name);
        Ok(())
    }
}

impl UpdateCommand {
    pub fn run(&self) -> Result<()> {
        let path = self.file.as_ref().map(Path::new);
        let mut config = FlowrsConfig::from_file(path)?;

        let name: String = if self.name.is_none() {
            Select::new(
                "name",
                config
                    .servers
                    .iter()
                    .map(|server| server.name.clone())
                    .collect(),
            )
            .prompt()?
        } else {
            self.name.clone().unwrap()
        };
        let airflow_config: &mut AirflowConfig = config
            .servers
            .iter_mut()
            .find(|server| server.name == name)
            .expect("🤔 Airflow config not found ...");

        let name = inquire::Text::new("name")
            .with_default(&airflow_config.name)
            .prompt()?;
        let endpoint = inquire::Text::new("endpoint")
            .with_default(&airflow_config.endpoint)
            .with_validator(validate_endpoint)
            .prompt()?;

        let auth_type =
            Select::new("authentication type", ConfigOption::iter().collect()).prompt()?;

        airflow_config.name = name;
        airflow_config.endpoint = endpoint;
        match auth_type {
            ConfigOption::BasicAuth => {
                let username = inquire::Text::new("username").prompt()?;
                let password = inquire::Password::new("password")
                    .with_display_toggle_enabled()
                    .prompt()?;

                airflow_config.auth = AirflowAuth::BasicAuth(BasicAuth { username, password });
            }
            ConfigOption::Token(_) => {
                let cmd = Some(inquire::Text::new("cmd").prompt()?);
                let token: String;
                if let Some(cmd) = &cmd {
                    info!("🔑 Running command: {}", cmd);
                    let output = std::process::Command::new(cmd)
                        .output()
                        .expect("failed to execute process");
                    token = String::from_utf8(output.stdout)?;
                } else {
                    token = inquire::Text::new("token").prompt()?;
                }
                airflow_config.auth = AirflowAuth::TokenAuth(TokenCmd { cmd, token });
            }
        };

        write_config(&config, path)?;

        println!("✅ Config updated successfully!");
        Ok(())
    }
}

fn validate_endpoint(
    endpoint: &str,
) -> std::result::Result<Validation, Box<dyn std::error::Error + Send + Sync>> {
    match Url::parse(endpoint) {
        Ok(_) => Ok(Validation::Valid),
        Err(error) => Ok(Validation::Invalid(error.into())),
    }
}

fn write_config(config: &FlowrsConfig, path: Option<&Path>) -> Result<()> {
    let path = match path {
        Some(path) => path,
        None => CONFIG_FILE.as_path(),
    };
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)?;

    file.write_all(toml::to_string(config)?.as_bytes())?;
    Ok(())
}
