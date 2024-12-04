use std::path::Path;

use inquire::Select;
use log::info;
use strum::IntoEnumIterator;

use super::model::AddCommand;
use crate::{
    airflow::config::{AirflowAuth, AirflowConfig, BasicAuth, FlowrsConfig, TokenCmd},
    commands::config::model::{validate_endpoint, ConfigOption},
};
use anyhow::Result;

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
                    managed: None,
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
                    auth: AirflowAuth::TokenAuth(TokenCmd {
                        cmd,
                        token: Some(token),
                    }),
                    managed: None,
                }
            }
        };

        let path = self.file.as_ref().map(Path::new);
        let mut config = FlowrsConfig::from_file(path)?;

        if let Some(mut servers) = config.servers.clone() {
            servers.retain(|server| server.name != new_config.name && server.managed.is_none());
            servers.push(new_config);
            config.servers = Some(servers);
        } else {
            config.servers = Some(vec![new_config]);
        }

        config.to_file(path)?;

        println!("✅ Config added successfully!");
        Ok(())
    }
}
