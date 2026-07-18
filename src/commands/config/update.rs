use std::path::PathBuf;

use inquire::Select;
use log::info;
use strum::IntoEnumIterator;

use super::model::UpdateCommand;
use crate::commands::config::model::{validate_endpoint, ConfigOption};
use flowrs_config::{AirflowAuth, AirflowConfig, BasicAuth, FlowrsConfig, TokenSource};

use anyhow::{anyhow, Context, Result};

impl UpdateCommand {
    pub fn run(&self) -> Result<()> {
        let path = self.file.as_ref().map(PathBuf::from);
        let mut config = FlowrsConfig::from_file(path.as_ref(), &crate::CONFIG_PATHS)?;

        if config.servers.is_empty() {
            println!("❌ No servers found in config file");
            return Ok(());
        }

        let mut servers = config.servers;

        let name: String = match &self.name {
            Some(name) => name.clone(),
            None => Select::new(
                "name",
                servers.iter().map(|server| server.name.clone()).collect(),
            )
            .prompt()?,
        };

        let index = find_server(&servers, &name)?;
        let airflow_config: &mut AirflowConfig = &mut servers[index];

        let name = inquire::Text::new("name")
            .with_default(&airflow_config.name)
            .prompt()?;
        let endpoint = inquire::Text::new("endpoint")
            .with_default(&airflow_config.endpoint)
            .with_validator(validate_endpoint)
            .prompt()?;

        let insecure = if self.insecure || airflow_config.insecure {
            inquire::Confirm::new("Allow insecure SSL connections? (danger)")
                .with_help_message("Disables TLS certificate verification (MITM risk). Use only for local/dev port-forwarded endpoints.")
                .with_default(airflow_config.insecure)
                .prompt()?
        } else {
            false
        };

        let auth_type =
            Select::new("authentication type", ConfigOption::iter().collect()).prompt()?;

        airflow_config.name = name;
        airflow_config.endpoint = endpoint;
        airflow_config.insecure = insecure;
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
                    .with_context(|| format!("failed to run token command: {cmd}"))?;
                // Validate the command produces a token
                let _token = String::from_utf8(output.stdout)?;
                airflow_config.auth = AirflowAuth::Token(TokenSource::Command { cmd });
            }
        }

        config.servers = servers;
        config.write_to_file(&crate::CONFIG_PATHS)?;

        println!("✅ Config updated successfully!");
        Ok(())
    }
}

fn find_server(servers: &[AirflowConfig], name: &str) -> Result<usize> {
    servers
        .iter()
        .position(|server| server.name == name)
        .ok_or_else(|| anyhow!("no server named '{name}' found in config"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use flowrs_config::{AirflowVersion, BasicAuth};

    fn server(name: &str) -> AirflowConfig {
        AirflowConfig {
            name: name.to_string(),
            endpoint: "http://localhost:8080".to_string(),
            auth: AirflowAuth::Basic(BasicAuth {
                username: "airflow".to_string(),
                password: "airflow".to_string(),
            }),
            managed: None,
            version: AirflowVersion::V2,
            timeout_secs: 30,
            insecure: false,
        }
    }

    #[test]
    fn find_server_locates_known_name() {
        let servers = vec![server("dev"), server("prod")];
        assert_eq!(find_server(&servers, "prod").unwrap(), 1);
    }

    #[test]
    fn find_server_errors_on_unknown_name() {
        let servers = vec![server("dev")];
        assert!(find_server(&servers, "typo").is_err());
    }
}
