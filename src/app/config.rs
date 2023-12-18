use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::app::error::Result;
use crate::CONFIG_FILE;

#[derive(Deserialize, Serialize, Debug)]
pub struct FlowrsConfig {
    pub servers: Vec<AirflowConfig>,
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
    pub token: String,
}
// Maye use a trait instead? Something that returns an Airflow Client?

impl FlowrsConfig {
    pub fn from_file(config_path: Option<&Path>) -> Result<Self> {
            let path = match config_path {
        Some(path) => path,
        None => CONFIG_FILE.as_path(),
    };

    let toml_read = std::fs::read_to_string(path);
    if let Ok(toml_config) = toml_read {
        toml::from_str(&toml_config).map_err(|e| e.into())
    } else {
        Ok(FlowrsConfig { servers: vec![] })
    }
    }
}


#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::app::config::FlowrsConfig;


    #[test]
    fn test_get_config() {
        let result = FlowrsConfig::from_file(Some(Path::new(".flowrs"))).unwrap();
        assert_eq!(result.servers.len(), 2);
        assert_eq!(result.servers[0].name, "test");
    }
}
