use crate::airflow::config::{AirflowAuth, AirflowConfig, ManagedService};
use crate::app::error::Result;
use log::debug;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MwaaEnvironment {
    pub name: String,
    #[serde(rename = "clusterName")]
    pub cluster_name: String,
    #[serde(rename = "tenantId")]
    pub tenant_id: String,
    #[serde(rename = "airflowVersion")]
    pub airflow_version: String,
}

pub async fn list_mwaa_environments() -> Result<Vec<String>> {
    debug!("Listing MWAA environments");
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;

    let client = aws_sdk_mwaa::Client::new(&config);
    let resp = client.list_environments().send().await?;
    Ok(resp.environments)
}

pub async fn get_mwaa_environment_servers() -> Result<Vec<AirflowConfig>> {
    let environments = list_mwaa_environments().await?;

    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let client = aws_sdk_mwaa::Client::new(&config);
    let mwaa_env_futures = environments
        .iter()
        .map(|env| tokio::spawn(client.get_environment().set_name(Some(env.clone())).send()));

    let mwaa_envs: Vec<AirflowConfig> = futures::future::join_all(mwaa_env_futures)
        .await
        .into_iter()
        .filter_map(|env| {
            env.expect("Failed to call MWAA environment API")
                .expect("Failed to unwrap MWAA environment GET response")
                .environment
        })
        .map(|env| AirflowConfig {
            name: env.name.expect("Failed to get MWAA environment name"),
            endpoint: env
                .webserver_url
                .expect("Failed to get MWAA environment webserver URL"),
            auth: AirflowAuth::Session,
            managed: Some(ManagedService::Mwaa),
        })
        .collect();

    Ok(mwaa_envs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_mwaa_environments() {
        let envs = list_mwaa_environments().await.unwrap();
        assert!(!envs.is_empty());
    }
}
