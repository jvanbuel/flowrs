pub mod paths;

use std::fmt::{Display, Formatter};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use clap::ValueEnum;
use log::info;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use super::managed_services::astronomer::get_astronomer_environment_servers;
use super::managed_services::conveyor::get_conveyor_environment_servers;
use super::managed_services::mwaa::get_mwaa_environment_servers;
use crate::CONFIG_PATHS;
use anyhow::Result;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Default)]
pub enum AirflowVersion {
    #[default]
    V2,
    V3,
}

impl AirflowVersion {
    pub const fn api_path(&self) -> &str {
        match self {
            Self::V2 => "api/v1",
            Self::V3 => "api/v2",
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, ValueEnum, EnumIter)]
pub enum ManagedService {
    Conveyor,
    Mwaa,
    Astronomer,
    Gcc,
}

impl Display for ManagedService {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Conveyor => write!(f, "Conveyor"),
            Self::Mwaa => write!(f, "MWAA"),
            Self::Astronomer => write!(f, "Astronomer"),
            Self::Gcc => write!(f, "Google Cloud Composer"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FlowrsConfig {
    #[serde(default)]
    pub servers: Vec<AirflowConfig>,
    #[serde(default)]
    pub managed_services: Vec<ManagedService>,
    pub active_server: Option<String>,
    #[serde(skip_serializing)]
    pub path: Option<PathBuf>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AirflowConfig {
    pub name: String,
    pub endpoint: String,
    pub auth: crate::airflow::config::AirflowAuth,
    pub managed: Option<ManagedService>,
    #[serde(default)]
    pub version: AirflowVersion,
    /// Request timeout in seconds. Defaults to 30 seconds if not specified.
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

const fn default_timeout() -> u64 {
    30
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum AirflowAuth {
    Basic(BasicAuth),
    Token(TokenSource),
    Conveyor,
    Mwaa(super::managed_services::mwaa::MwaaAuth),
    Astronomer(super::managed_services::astronomer::AstronomerAuth),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum TokenSource {
    Command { cmd: String },
    Static { token: String },
}

impl Default for FlowrsConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowrsConfig {
    /// Creates a new `FlowrsConfig` with default values.
    ///
    /// Returns a `FlowrsConfig` with:
    /// - No servers configured
    /// - No managed services
    /// - No active server
    /// - Default config file path
    pub fn new() -> Self {
        Self {
            servers: Vec::new(),
            managed_services: Vec::new(),
            active_server: None,
            path: Some(CONFIG_PATHS.write_path.clone()),
        }
    }

    pub fn from_file(config_path: Option<&PathBuf>) -> Result<Self> {
        let path = config_path
            .filter(|p| p.exists())
            .cloned()
            .unwrap_or_else(|| {
                // No valid path was provided by the user, use the default read path
                let default_path = CONFIG_PATHS.read_path.clone();
                info!("Using configuration path: {}", default_path.display());
                default_path
            });

        // If no config at the default path, return an empty (default) config
        let toml_config = std::fs::read_to_string(&path).unwrap_or_default();
        let mut config = Self::parse_toml(&toml_config)?;
        config.path = Some(path);
        Ok(config)
    }

    pub fn parse_toml(config: &str) -> Result<Self> {
        let config: Self = toml::from_str(config)?;
        info!(
            "Loaded config: servers={}, managed_services={}",
            config.servers.len(),
            config.managed_services.len()
        );
        Ok(config)
    }

    fn extend_servers<I>(&mut self, new_servers: I)
    where
        I: IntoIterator<Item = AirflowConfig>,
    {
        self.servers.extend(new_servers);
    }

    /// Expands the config by resolving managed services and adding their servers.
    /// This is an async convenience function that should be called after `from_file`/`from_str`
    /// when you need to resolve managed service environments.
    /// Returns a tuple of (config, errors) where errors contains any non-fatal errors encountered.
    pub async fn expand_managed_services(mut self) -> Result<(Self, Vec<String>)> {
        let mut all_errors = Vec::new();

        if self.managed_services.is_empty() {
            return Ok((self, all_errors));
        }

        let services = self.managed_services.clone();
        for service in services {
            match service {
                ManagedService::Conveyor => {
                    let conveyor_servers = get_conveyor_environment_servers()?;
                    self.extend_servers(conveyor_servers);
                }
                ManagedService::Mwaa => {
                    let mwaa_servers = get_mwaa_environment_servers().await?;
                    self.extend_servers(mwaa_servers);
                }
                ManagedService::Astronomer => {
                    let (astronomer_servers, errors) = get_astronomer_environment_servers().await;
                    all_errors.extend(errors);
                    self.extend_servers(astronomer_servers);
                }
                ManagedService::Gcc => {
                    log::warn!("ManagedService::Gcc (Google Cloud Composer) expansion not implemented; skipping");
                }
            }
        }
        info!(
            "Expanded config: servers={}, errors={}",
            self.servers.len(),
            all_errors.len()
        );
        Ok((self, all_errors))
    }

    pub fn to_str(&self) -> Result<String> {
        toml::to_string(self).map_err(std::convert::Into::into)
    }

    pub fn write_to_file(&mut self) -> Result<()> {
        let path = self
            .path
            .clone()
            .unwrap_or_else(|| CONFIG_PATHS.write_path.clone());

        // Create parent directory if it doesn't exist (for XDG path)
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)?;

        // Only write non-managed servers to the config file
        self.servers.retain(|server| server.managed.is_none());
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

        [servers.auth.Basic]
        username = "airflow"
        password = "airflow"
        "#;

    #[test]
    fn test_get_config() {
        let result = FlowrsConfig::parse_toml(TEST_CONFIG).unwrap();
        assert_eq!(result.servers.len(), 1);
        assert_eq!(result.servers[0].name, "test");
    }

    const TEST_CONFIG_CONVEYOR: &str = r#"
managed_services = ["Conveyor"]

[[servers]]
name = "bla"
endpoint = "http://localhost:8080"
version = "V2"
timeout_secs = 30

[servers.auth.Basic]
username = "airflow"
password = "airflow"
    "#;
    #[test]
    fn test_get_config_conveyor() {
        let result = FlowrsConfig::parse_toml(TEST_CONFIG_CONVEYOR.trim()).unwrap();
        assert_eq!(result.managed_services.len(), 1);
        assert_eq!(result.managed_services[0], ManagedService::Conveyor);
    }

    #[test]
    fn test_write_config_conveyor() {
        let config = FlowrsConfig {
            servers: vec![AirflowConfig {
                name: "bla".to_string(),
                endpoint: "http://localhost:8080".to_string(),
                auth: AirflowAuth::Basic(BasicAuth {
                    username: "airflow".to_string(),
                    password: "airflow".to_string(),
                }),
                managed: None,
                version: AirflowVersion::V2,
                timeout_secs: default_timeout(),
            }],
            managed_services: vec![ManagedService::Conveyor],
            active_server: None,
            path: None,
        };

        let serialized_config = config.to_str().unwrap();
        assert_eq!(serialized_config.trim(), TEST_CONFIG_CONVEYOR.trim());
    }

    #[test]
    fn non_existing_path() {
        let path = PathBuf::from("non-existing.toml");
        let config = FlowrsConfig::from_file(Some(&path));
        assert!(config.is_ok());

        let config = config.unwrap();
        assert!(config.path.is_some());
    }

    #[test]
    fn none_path() {
        let config = FlowrsConfig::from_file(None);
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.path.unwrap(), CONFIG_PATHS.read_path);
    }
}
