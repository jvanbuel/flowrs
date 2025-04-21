use crate::airflow::config::{AirflowAuth, AirflowConfig, ManagedService};
use anyhow::Result;
use log::info;
use serde::{Deserialize, Serialize};

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
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg("conveyor auth get --quiet")
        .output()?;
    let token = serde_json::from_str::<ConveyorTokenResponse>(&String::from_utf8(output.stdout)?)?
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

    #[tokio::test]
    async fn test_get_conveyor_token() {
        let token = get_conveyor_token().unwrap();
        assert!(!token.is_empty());
    }
}
