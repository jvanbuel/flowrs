use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub servers: Vec<AirflowConfig>,
}

#[derive(Deserialize)]
pub struct AirflowConfig {
    pub name: String,
    pub endpoint: String,
    pub token: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

pub fn get_config() -> Config {
    let file = std::fs::read_to_string(".flowrs").unwrap();
    toml::from_str(&file).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::app::auth::get_config;

    #[test]
    fn test_get_config() {
        let result = get_config();
        assert_eq!(result.servers.len(), 2);
        assert_eq!(result.servers[1].name, "test");
    }
}
