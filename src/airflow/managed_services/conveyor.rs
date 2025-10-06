use crate::airflow::config::{AirflowAuth, AirflowConfig, ManagedService};
use anyhow::{Context, Result};
use dirs::home_dir;
use expectrl::spawn;
use log::info;
use serde::{Deserialize, Serialize};
use std::io::Read;

// New ConveyorClient struct
#[derive(Debug, Clone)]
pub struct ConveyorClient {}

impl ConveyorClient {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_token(&self) -> Result<String> {
        // Use expectrl to spawn the command in a pseudo-terminal
        let mut session = spawn("conveyor auth get --quiet")
            .context("Failed to spawn conveyor auth get command")?;

        // Create a buffer to read the output into
        let mut output_bytes = Vec::new();

        // Read all output until EOF into the buffer
        session
            .read_to_end(&mut output_bytes)
            .context("Failed to read output from conveyor auth get")?;

        let token = serde_json::from_str::<ConveyorTokenResponse>(
            &String::from_utf8(output_bytes).context("Failed to decode output as UTF-8")?,
        )
        .context("Failed to parse JSON token from conveyor output")?
        .access_token;

        Ok(token)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ConveyorEnvironment {
    pub name: String,
    #[serde(rename = "clusterName")]
    pub cluster_name: String,
    #[serde(rename = "tenantId")]
    pub tenant_id: String,
    #[serde(rename = "airflowVersion")]
    pub airflow_version: String,
}

pub fn list_conveyor_environments() -> Result<Vec<ConveyorEnvironment>> {
    // Use the new ConveyorClient to authenticate
    let conveyor_client = ConveyorClient::new();
    conveyor_client.get_token()?; // Ensure authentication before listing environments

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg("conveyor environment list -o json")
        .output()?;
    let environments: Vec<ConveyorEnvironment> =
        serde_json::from_str(&String::from_utf8(output.stdout)?)?;
    info!("Conveyor Environments: {:?}", environments);
    Ok(environments)
}

pub fn get_conveyor_environment_servers() -> Result<Vec<AirflowConfig>> {
    let environments = list_conveyor_environments()?;
    let api_endpoint = get_conveyor_api_endpoint()?;

    let servers = environments
        .iter()
        .map(|env| AirflowConfig {
            name: env.name.clone(),
            endpoint: format!("{}/environments/{}/airflow/", api_endpoint, env.name),
            auth: AirflowAuth::Conveyor,
            managed: Some(ManagedService::Conveyor),
        })
        .collect();
    Ok(servers)
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ConveyorTokenResponse {
    pub access_token: String,
}

#[derive(Deserialize, Debug)]
struct ConveyorProfiles {
    activeprofile: String,
    #[serde(rename = "version")]
    _version: Option<i8>,
    #[serde(flatten)]
    profiles: std::collections::HashMap<String, ConveyorProfile>,
}

#[derive(Deserialize, Debug)]
struct ConveyorProfile {
    api: String,
}

fn get_conveyor_api_endpoint() -> Result<String> {
    let profiles_path = home_dir()
        .context("Could not determine home directory")?
        .join(".conveyor/profiles.toml");

    let profiles_content = std::fs::read_to_string(&profiles_path)
        .context("Failed to read ~/.conveyor/profiles.toml")?;

    let profiles_config: ConveyorProfiles =
        toml::from_str(&profiles_content).context("Failed to parse profiles.toml")?;

    let active_profile = profiles_config
        .profiles
        .get(&profiles_config.activeprofile)
        .context(format!(
            "Active profile '{}' not found in profiles.toml",
            profiles_config.activeprofile
        ))?;

    Ok(active_profile.api.clone())
}

// Removed the standalone get_conveyor_token function as its logic is now in ConveyorClient::get_token

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airflow::config::FlowrsConfig;

    const TEST_CONFIG: &str = r#"[[servers]]
        name = "test"
        endpoint = "http://localhost:8080"
        auth = { BasicAuth = { username = "airflow", password = "airflow" } }
        "#;

    #[tokio::test]
    async fn test_list_conveyor_environments() {
        let config: FlowrsConfig = toml::from_str(str::trim(TEST_CONFIG)).unwrap();
        let _server = config.servers.unwrap()[0].clone();
        let environments = get_conveyor_environment_servers().unwrap();
        assert!(!environments.is_empty());
    }

    #[test]
    fn test_get_conveyor_token() {
        // Test the new client method
        let client = ConveyorClient::new();
        let token = client.get_token().unwrap();
        assert!(!token.is_empty());
    }
}
