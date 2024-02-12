use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use clap::Parser;
use inquire::validator::Validation;
use inquire::Select;
use inquire::Text;
use strum::Display;
use url::Url;

use crate::app::auth::AirflowConfig;
use crate::app::auth::Config;
use crate::app::error::Result;
use crate::CONFIG_FILE;
use strum::{IntoEnumIterator, EnumIter};

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

        let mut token = None;
        let mut username = None;
        let mut password = None;

        let auth_type =
            Select::new("authentication type", ConfigOption::iter().collect()).prompt()?;

        match auth_type {
            ConfigOption::BasicAuth => {
                username = Some(inquire::Text::new("username").prompt()?);
                password = Some(
                    inquire::Password::new("password")
                        .with_display_toggle_enabled()
                        .prompt()?,
                );
            },
            ConfigOption::Token(cmd) => {
                token = match cmd {
                    Some(cmd) => Some(cmd), // TODO: get token from command, return 
                    None => Some(Text::new("token").prompt()?),
                }
            },
            
        }

        let airflow_config = AirflowConfig {
            name,
            endpoint,
            username,
            password,
            token,
        };

        let path = self.file.as_ref().map(Path::new);
        let mut config = crate::app::auth::get_config(path)?;
        config
            .servers
            .retain(|server| server.name != airflow_config.name);

        config.servers.push(airflow_config);

        write_config(&config, path)?;

        println!("✅ Config added successfully!");
        Ok(())
    }
}

impl RemoveCommand {
    pub fn run(&self) -> Result<()> {
        let path = self.file.as_ref().map(Path::new);
        let mut config = crate::app::auth::get_config(path)?;

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
        let mut config = crate::app::auth::get_config(path)?;

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

        let mut token = None;
        let mut username = None;
        let mut password = None;

        let auth_type =
            Select::new("authentication type", vec!["username/password", "token"]).prompt()?;

        if let "username/password" = auth_type {
            username = Some(inquire::Text::new("username").prompt()?);
            password = Some(inquire::Text::new("password").prompt()?);
        } else {
            token = Some(Text::new("token").prompt()?);
        }

        airflow_config.name = name;
        airflow_config.endpoint = endpoint;
        airflow_config.username = username;
        airflow_config.password = password;
        airflow_config.token = token;

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

fn write_config(config: &Config, path: Option<&Path>) -> Result<()> {
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
