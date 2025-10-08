use std::path::PathBuf;

use inquire::Select;
use log::info;
use strum::IntoEnumIterator;

use super::model::AddCommand;
use crate::{
    airflow::config::{AirflowAuth, AirflowConfig, AirflowVersion, BasicAuth, FlowrsConfig, TokenCmd},
    commands::config::model::{validate_endpoint, ConfigOption},
};
use anyhow::Result;

impl AddCommand {
    pub fn run(&self) -> Result<()> {
        let name = inquire::Text::new("name").prompt()?;
        let endpoint = inquire::Text::new("endpoint")
            .with_validator(validate_endpoint)
            .prompt()?;

        let version_str = inquire::Select::new(
            "Airflow version",
            vec!["v2", "v3"],
        )
        .with_help_message("Select the Airflow API version")
        .prompt()?;

        let version = match version_str {
            "v2" => AirflowVersion::V2,
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
                    auth: AirflowAuth::Token(TokenCmd {
                        cmd,
                        token: Some(token),
                    }),
                    managed: None,
                    version,
                }
            }
        };

        let path = self.file.as_deref().map(PathBuf::from);
        let mut config = FlowrsConfig::from_file(&path)?;

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
