use std::path::Path;

use inquire::Select;
use log::info;
use strum::IntoEnumIterator;

use super::model::UpdateCommand;
use crate::{
    airflow::config::{AirflowAuth, AirflowConfig, BasicAuth, FlowrsConfig, TokenCmd},
    commands::config::model::{validate_endpoint, ConfigOption},
};

use anyhow::Result;

impl UpdateCommand {
    pub fn run(&self) -> Result<()> {
        let path = self.file.as_ref().map(Path::new);
        let mut config = FlowrsConfig::from_file(path)?;

        if config.servers.is_none() {
            println!("❌ No servers found in config file");
            return Ok(());
        }

        let mut servers = config.servers.unwrap();

        let name: String = if self.name.is_none() {
            Select::new(
                "name",
                servers.iter().map(|server| server.name.clone()).collect(),
            )
            .prompt()?
        } else {
            self.name.clone().unwrap()
        };

        let airflow_config: &mut AirflowConfig = servers
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
                airflow_config.auth = AirflowAuth::TokenAuth(TokenCmd {
                    cmd,
                    token: Some(token),
                });
            }
        };

        config.servers = Some(servers);
        config.to_file(path)?;

        println!("✅ Config updated successfully!");
        Ok(())
    }
}
