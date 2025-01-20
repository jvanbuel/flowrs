use std::fmt::{Display, Formatter};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use clap::ValueEnum;
use log::info;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use super::managed_services::conveyor::get_conveyor_environment_servers;
use crate::CONFIG_FILE;
use anyhow::Result;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, ValueEnum, EnumIter)]
pub enum ManagedService {
    Conveyor,
    Mwaa,
    Astronomer,
    Gcc,
}

impl Display for ManagedService {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ManagedService::Conveyor => write!(f, "Conveyor"),
            ManagedService::Mwaa => write!(f, "MWAA"),
            ManagedService::Astronomer => write!(f, "Astronomer"),
            ManagedService::Gcc => write!(f, "Google Cloud Composer"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FlowrsConfig {
    pub servers: Option<Vec<AirflowConfig>>,
    pub managed_services: Option<Vec<ManagedService>>,
    pub active_server: Option<String>,
    #[serde(skip_serializing)]
    pub path: Option<PathBuf>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AirflowConfig {
    pub name: String,
    pub endpoint: String,
    pub auth: AirflowAuth,
    pub managed: Option<ManagedService>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum AirflowAuth {
    BasicAuth(BasicAuth),
    TokenAuth(TokenCmd),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TokenCmd {
    pub cmd: Option<String>,
    pub token: Option<String>,
}

impl Default for FlowrsConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowrsConfig {
    /// Creates a new FlowrsConfig with default values.
    ///
    /// Returns a FlowrsConfig with:
    /// - No servers configured
    /// - No managed services
    /// - No active server
    /// - Default config file path
    pub fn new() -> Self {
        Self {
            servers: None,
            managed_services: None,
            active_server: None,
            path: Some(CONFIG_FILE.as_path().to_path_buf()),
        }
    }
    pub fn from_file(config_path: &Option<PathBuf>) -> Result<Self> {
        let path = config_path
            .clone()
            .filter(|p| p.exists())
            .unwrap_or_else(|| CONFIG_FILE.as_path().to_path_buf());

        // If no config at the default path, return an empty (default) config
        let toml_config = std::fs::read_to_string(&path).unwrap_or_default();
        let mut config = Self::from_str(&toml_config)?;
        config.path = Some(path.to_path_buf());
        Ok(config)
    }
    pub fn from_str(config: &str) -> Result<Self> {
        let mut config: FlowrsConfig = toml::from_str(config)?;

        info!("Config: {:?}", config);

        // This should not happen here, but when the App has started
        if config.managed_services.is_none() {
            Ok(config)
        } else {
            let services = config.managed_services.clone().unwrap();
            for service in services {
                match service {
                    ManagedService::Conveyor => {
                        let conveyor_servers = get_conveyor_environment_servers()?;
                        if config.servers.is_none() {
                            config.servers = Some(conveyor_servers);
                        } else {
                            let mut existing_servers = config.servers.clone().unwrap();
                            existing_servers.extend(conveyor_servers);
                            config.servers = Some(existing_servers);
                        }
                    }
                    ManagedService::Mwaa => {
                        todo!();
                    }
                    ManagedService::Astronomer => {
                        todo!();
                    }
                    ManagedService::Gcc => {
                        todo!();
                    }
                }
            }
            info!("Updated Config: {:?}", config);
            Ok(config)
        }
    }

    pub fn to_str(&self) -> Result<String> {
        toml::to_string(self).map_err(|e| e.into())
    }

    pub fn write_to_file(&mut self) -> Result<()> {
        let path = self
            .path
            .clone()
            .unwrap_or(CONFIG_FILE.as_path().to_path_buf());
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)?;

        // Only write non-managed servers to the config file
        if let Some(servers) = &mut self.servers {
            *servers = servers
                .iter()
                .filter(|server| server.managed.is_none())
                .cloned()
                .collect();
        }
        file.write_all(Self::to_str(self)?.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_CONFIG: &str = r#"[[servers]]
        name = "test"
        endpoint = "http://localhost:8080"

        [servers.auth.BasicAuth]
        username = "airflow"
        password = "airflow"
        "#;

    #[test]
    fn test_get_config() {
        let result = FlowrsConfig::from_str(TEST_CONFIG).unwrap();
        let servers = result.servers.unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "test");
    }

    const TEST_CONFIG_CONVEYOR: &str = r#"managed_services = ["Conveyor"]

[[servers]]
name = "bla"
endpoint = "http://localhost:8080"

[servers.auth.BasicAuth]
username = "airflow"
password = "airflow"
    "#;
    #[test]
    fn test_get_config_conveyor() {
        let result = FlowrsConfig::from_str(TEST_CONFIG_CONVEYOR).unwrap();
        let services = result.managed_services.unwrap();
        assert_eq!(services.len(), 1);
        assert_eq!(services[0], ManagedService::Conveyor);
    }

    #[test]
    fn test_write_config_conveyor() {
        let config = FlowrsConfig {
            servers: Some(vec![AirflowConfig {
                name: "bla".to_string(),
                endpoint: "http://localhost:8080".to_string(),
                auth: AirflowAuth::BasicAuth(BasicAuth {
                    username: "airflow".to_string(),
                    password: "airflow".to_string(),
                }),
                managed: None,
            }]),
            managed_services: Some(vec![ManagedService::Conveyor]),
            active_server: None,
            path: None,
        };

        let serialized_config = config.to_str().unwrap();
        assert_eq!(serialized_config.trim(), TEST_CONFIG_CONVEYOR.trim());
    }

    #[test]
    fn non_existing_path() {
        let config = FlowrsConfig::from_file(&Some(PathBuf::from("non-existing.toml")));
        assert!(config.is_ok());

        let config = config.unwrap();
        assert!(config.path.is_some());
    }

    #[test]
    fn none_path() {
        let config = FlowrsConfig::from_file(&None);
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.path.unwrap(), CONFIG_FILE.as_path().to_path_buf());
    }
}
