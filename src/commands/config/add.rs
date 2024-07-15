use std::path::Path;

use inquire::Select;
use log::info;
use strum::IntoEnumIterator;

use super::model::AddCommand;
use crate::{
    app::{
        config::{AirflowAuth, AirflowConfig, BasicAuth, FlowrsConfig, TokenCmd},
        error::Result,
    },
    commands::config::{model::ConfigOption, validate_endpoint, write_config},
};

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
                    auth: AirflowAuth::TokenAuth(TokenCmd {
                        cmd,
                        token: Some(token),
                    }),
                }
            }
        };

        let path = self.file.as_ref().map(Path::new);
        let mut config = FlowrsConfig::from_file(path)?;

        if let Some(mut servers) = config.servers.clone() {
            servers.retain(|server| server.name != new_config.name);
            servers.push(new_config);
            config.servers = Some(servers);
        } else {
            config.servers = Some(vec![new_config]);
        }

        write_config(&config, path)?;

        println!("✅ Config added successfully!");
        Ok(())
    }
}
