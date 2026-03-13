pub mod managed_auth;
pub mod paths;

use std::fmt::{Display, Formatter};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use clap::ValueEnum;
use log::info;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use anyhow::Result;
use paths::ConfigPaths;

use managed_auth::{AstronomerAuth, ComposerAuth, MwaaAuth};

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

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct GccConfig {
    pub regions: Vec<String>,
    /// GCP project IDs to search for Composer environments.
    /// `None` means search all accessible projects.
    pub projects: Option<Vec<String>>,
}

const TICK_RATE_MS: u64 = 200;
const MIN_POLL_INTERVAL_MS: u64 = 500;

const fn default_poll_interval_ms() -> u64 {
    2000
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FlowrsConfig {
    #[serde(default)]
    pub servers: Vec<AirflowConfig>,
    #[serde(default)]
    pub managed_services: Vec<ManagedService>,
    pub active_server: Option<String>,
    /// API poll interval in milliseconds. Controls how often the TUI refreshes
    /// data from the Airflow API. Minimum 500ms, default 2000ms.
    #[serde(default = "default_poll_interval_ms")]
    pub poll_interval_ms: u64,
    #[serde(default)]
    pub gcc: Option<GccConfig>,
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
    Mwaa(MwaaAuth),
    Astronomer(AstronomerAuth),
    Composer(ComposerAuth),
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

impl FlowrsConfig {
    /// Creates a new `FlowrsConfig` with default values.
    ///
    /// Returns a `FlowrsConfig` with:
    /// - No servers configured
    /// - No managed services
    /// - No active server
    /// - Default config file path derived from the provided `ConfigPaths`
    pub fn new(config_paths: &ConfigPaths) -> Self {
        Self {
            servers: Vec::new(),
            managed_services: Vec::new(),
            active_server: None,
            poll_interval_ms: default_poll_interval_ms(),
            gcc: None,
            path: Some(config_paths.write_path.clone()),
        }
    }

    /// Compute the tick multiplier for API polling based on `poll_interval_ms`.
    /// Clamps values below the minimum and logs a warning.
    pub fn poll_tick_multiplier(&self) -> u32 {
        let interval = if self.poll_interval_ms < MIN_POLL_INTERVAL_MS {
            log::warn!(
                "poll_interval_ms ({}) is below minimum ({}), clamping",
                self.poll_interval_ms,
                MIN_POLL_INTERVAL_MS
            );
            MIN_POLL_INTERVAL_MS
        } else {
            self.poll_interval_ms
        };
        #[allow(clippy::cast_possible_truncation)]
        let multiplier = (interval / TICK_RATE_MS) as u32;
        multiplier.max(1)
    }

    pub fn from_file(config_path: Option<&PathBuf>, config_paths: &ConfigPaths) -> Result<Self> {
        let path = config_path
            .filter(|p| p.exists())
            .cloned()
            .unwrap_or_else(|| {
                // No valid path was provided by the user, use the default read path
                let default_path = config_paths.read_path.clone();
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

    pub fn extend_servers<I>(&mut self, new_servers: I)
    where
        I: IntoIterator<Item = AirflowConfig>,
    {
        self.servers.extend(new_servers);
    }

    pub fn to_str(&self) -> Result<String> {
        toml::to_string(self).map_err(std::convert::Into::into)
    }

    pub fn write_to_file(&mut self, config_paths: &ConfigPaths) -> Result<()> {
        let path = self
            .path
            .clone()
            .unwrap_or_else(|| config_paths.write_path.clone());

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
poll_interval_ms = 2000

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
            poll_interval_ms: default_poll_interval_ms(),
            gcc: None,
            path: None,
        };

        let serialized_config = config.to_str().unwrap();
        assert_eq!(serialized_config.trim(), TEST_CONFIG_CONVEYOR.trim());
    }

    #[test]
    fn non_existing_path() {
        let config_paths = ConfigPaths::resolve();
        let path = PathBuf::from("non-existing.toml");
        let config = FlowrsConfig::from_file(Some(&path), &config_paths);
        assert!(config.is_ok());

        let config = config.unwrap();
        assert!(config.path.is_some());
    }

    #[test]
    fn none_path() {
        let config_paths = ConfigPaths::resolve();
        let config = FlowrsConfig::from_file(None, &config_paths);
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.path.unwrap(), config_paths.read_path);
    }

    #[test]
    fn test_poll_interval_ms_default() {
        let config = FlowrsConfig::parse_toml(TEST_CONFIG).unwrap();
        assert_eq!(config.poll_interval_ms, 2000);
        assert_eq!(config.poll_tick_multiplier(), 10);
    }

    #[test]
    fn test_poll_interval_ms_custom() {
        let toml = r#"
poll_interval_ms = 5000

[[servers]]
name = "test"
endpoint = "http://localhost:8080"

[servers.auth.Basic]
username = "airflow"
password = "airflow"
"#;
        let config = FlowrsConfig::parse_toml(toml).unwrap();
        assert_eq!(config.poll_interval_ms, 5000);
        assert_eq!(config.poll_tick_multiplier(), 25);
    }

    #[test]
    fn test_poll_interval_ms_clamped_to_minimum() {
        let toml = r#"
poll_interval_ms = 100

[[servers]]
name = "test"
endpoint = "http://localhost:8080"

[servers.auth.Basic]
username = "airflow"
password = "airflow"
"#;
        let config = FlowrsConfig::parse_toml(toml).unwrap();
        assert_eq!(config.poll_interval_ms, 100);
        // Should clamp to 500ms minimum → 500/200 = 2
        assert_eq!(config.poll_tick_multiplier(), 2);
    }
}
