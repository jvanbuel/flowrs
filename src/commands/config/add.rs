use std::path::PathBuf;

use inquire::Select;
use log::info;
use strum::IntoEnumIterator;

use super::model::AddCommand;
use crate::{
    airflow::config::{
        AirflowAuth, AirflowConfig, AirflowVersion, BasicAuth, FlowrsConfig, TokenSource,
    },
    commands::config::model::{validate_endpoint, ConfigOption},
};
use anyhow::{Context, Result};

impl AddCommand {
    pub fn run(&self) -> Result<()> {
        let name = inquire::Text::new("name").prompt()?;
        let endpoint = inquire::Text::new("endpoint")
            .with_validator(validate_endpoint)
            .prompt()?;

        let version_str = inquire::Select::new("Airflow version", vec!["v2", "v3"])
            .with_help_message("Select the Airflow API version")
            .prompt()?;

        let version = match version_str {
            "v3" => AirflowVersion::V3,
            _ => AirflowVersion::V2,
        };

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
                    auth: AirflowAuth::Basic(BasicAuth { username, password }),
                    managed: None,
                    version,
                    timeout_secs: 30,
                }
            }
            ConfigOption::Token(_) => {
                let cmd = inquire::Text::new("cmd").prompt()?;
                info!("🔑 Running command: {cmd}");
                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .output()
                    .with_context(|| format!("Failed to execute token command: {cmd}"))?;
                // Validate the command produces a token
                let _token = String::from_utf8(output.stdout)?.trim().to_string();

                AirflowConfig {
                    name,
                    endpoint,
                    auth: AirflowAuth::Token(TokenSource::Command { cmd }),
                    managed: None,
                    version,
                    timeout_secs: 30,
                }
            }
        };

        let path = self.file.as_ref().map(PathBuf::from);
        let mut config = FlowrsConfig::from_file(path.as_ref())?;

        // If the user provided a custom path, override the config path so write_to_file
        // uses the user-specified location even if it didn't exist during from_file
        if let Some(user_path) = path {
            config.path = Some(user_path);
        }

        if let Some(mut servers) = config.servers.clone() {
            servers.retain(|server| server.name != new_config.name && server.managed.is_none());
            servers.push(new_config);
            config.servers = Some(servers);
        } else {
            config.servers = Some(vec![new_config]);
        }

        config.write_to_file()?;

        println!("✅ Config added successfully!");
        Ok(())
    }
}
