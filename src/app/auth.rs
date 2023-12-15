use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::app::error::Result;
use crate::CONFIG_FILE;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub servers: Vec<AirflowConfig>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AirflowConfig {
    pub name: String,
    pub endpoint: String,
    pub token: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

pub fn get_config(config_path: Option<&Path>) -> Result<Config> {
    let path = match config_path {
        Some(path) => path,
        None => CONFIG_FILE.as_path(),
    };

    let toml_read = std::fs::read_to_string(path);
    if let Ok(toml_config) = toml_read {
        toml::from_str(&toml_config).map_err(|e| e.into())
    } else {
        Ok(Config { servers: vec![] })
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::app::auth::get_config;

    #[test]
    fn test_get_config() {
        let result = get_config(Some(Path::new(".flowrs"))).unwrap();
        assert_eq!(result.servers.len(), 2);
        assert_eq!(result.servers[1].name, "test");
    }
}
