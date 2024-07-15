use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::app::error::Result;
use crate::CONFIG_FILE;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum ManagedService {
    Conveyor,
    Mwaa,
    Astronomer,
    Gcc,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FlowrsConfig {
    pub servers: Option<Vec<AirflowConfig>>,
    pub managed_services: Option<Vec<ManagedService>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AirflowConfig {
    pub name: String,
    pub endpoint: String,
    pub auth: AirflowAuth,
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

impl FlowrsConfig {
    pub fn from_file(config_path: Option<&Path>) -> Result<Self> {
        let path = match config_path {
            Some(path) => path,
            None => CONFIG_FILE.as_path(),
        };

        let toml_config = std::fs::read_to_string(path)?;
        Self::from_str(&toml_config)
    }
    pub fn from_str(config: &str) -> Result<Self> {
        toml::from_str(config).map_err(|e| e.into())
    }

    pub fn to_str(&self) -> Result<String> {
        toml::to_string(self).map_err(|e| e.into())
    }

    pub fn to_file(self: FlowrsConfig, path: Option<&Path>) -> Result<()> {
        let path = match path {
            Some(path) => path,
            None => CONFIG_FILE.as_path(),
        };
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)?;

        file.write_all(Self::to_str(&self)?.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::app::config::*;

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

    const TEST_CONFIG_CONVEYOR: &str = r#"managed_services = ["Conveyor", "Mwaa"]"#;
    #[test]
    fn test_get_config_conveyor() {
        let result = FlowrsConfig::from_str(TEST_CONFIG_CONVEYOR).unwrap();
        let services = result.managed_services.unwrap();
        assert_eq!(services.len(), 2);
        assert_eq!(services[0], ManagedService::Conveyor);
    }

    #[test]
    fn test_write_config_conveyor() {
        let config = FlowrsConfig {
            servers: None,
            managed_services: Some(vec![ManagedService::Conveyor, ManagedService::Mwaa]),
        };

        let serialized_config = config.to_str().unwrap();
        assert_eq!(serialized_config.trim(), TEST_CONFIG_CONVEYOR);
    }
}
