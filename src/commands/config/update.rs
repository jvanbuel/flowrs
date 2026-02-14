use std::path::PathBuf;

use inquire::Select;
use log::info;
use strum::IntoEnumIterator;

use super::model::UpdateCommand;
use crate::{
    airflow::config::{AirflowAuth, AirflowConfig, BasicAuth, FlowrsConfig, TokenSource},
    commands::config::model::{validate_endpoint, ConfigOption},
};

use anyhow::Result;

impl UpdateCommand {
    pub fn run(&self) -> Result<()> {
        let path = self.file.as_ref().map(PathBuf::from);
        let mut config = FlowrsConfig::from_file(path.as_ref())?;

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

                airflow_config.auth = AirflowAuth::Basic(BasicAuth { username, password });
            }
            ConfigOption::Token(_) => {
                let cmd = inquire::Text::new("cmd").prompt()?;
                info!("🔑 Running command: {cmd}");
                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .output()
                    .expect("failed to execute process");
                // Validate the command produces a token
                let _token = String::from_utf8(output.stdout)?;
                airflow_config.auth = AirflowAuth::Token(TokenSource::Command { cmd });
            }
        }

        config.servers = Some(servers);
        config.write_to_file()?;

        println!("✅ Config updated successfully!");
        Ok(())
    }
}
