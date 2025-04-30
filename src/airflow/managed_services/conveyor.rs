use crate::airflow::config::{AirflowAuth, AirflowConfig, ManagedService};
use anyhow::{Context, Result};
use expectrl::spawn; // Import spawn function directly
use log::info;
use serde::{Deserialize, Serialize};
use std::io::Read; // Import the Read trait for read_to_end

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
    // Make sure we're authenticated; TODO: make this a bit cleaner e.g. by creating a ConveyorClient struct
    get_conveyor_token()?;
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
    let servers = environments
        .iter()
        .map(|env| AirflowConfig {
            name: env.name.clone(),
            endpoint: format!(
                "https://app.conveyordata.com/environments/{}/airflow/",
                env.name
            ),
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

pub fn get_conveyor_token() -> Result<String> {
    // Use expectrl to spawn the command in a pseudo-terminal
    let mut session = spawn("conveyor auth get --quiet") // Use imported spawn
        .context("Failed to spawn conveyor auth get command")?;

    // Create a buffer to read the output into
    let mut output_bytes = Vec::new();

    // Read all output until EOF into the buffer
    session
        .read_to_end(&mut output_bytes) // Pass buffer mutably
        .context("Failed to read output from conveyor auth get")?;

    let token = serde_json::from_str::<ConveyorTokenResponse>(
        &String::from_utf8(output_bytes).context("Failed to decode output as UTF-8")?,
    )
    .context("Failed to parse JSON token from conveyor output")?
    .access_token;

    Ok(token)
}

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
        let token = get_conveyor_token().unwrap();
        assert!(!token.is_empty());
    }
}
