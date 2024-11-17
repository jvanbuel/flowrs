use crate::airflow::config::{AirflowAuth, AirflowConfig, ManagedService, TokenCmd};
use crate::app::error::Result;
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
            auth: AirflowAuth::TokenAuth(TokenCmd {
                cmd: Some("conveyor auth get --quiet | jq -r .access_token".to_string()),
                token: None,
            }),
            managed: Some(ManagedService::Conveyor),
        })
        .collect();
    Ok(servers)
}
