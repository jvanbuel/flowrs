use anyhow::Result;
use async_trait::async_trait;
use reqwest::Method;

use crate::airflow::{model::common::DagList, traits::DagOperations};
use super::model;

use super::V2Client;

#[async_trait]
impl DagOperations for V2Client {
    async fn list_dags(&self) -> Result<DagList> {
        let r = self.base_api(Method::GET, "dags")?.build()?;
        let response = self.base.client.execute(r).await?.error_for_status()?;

        response
            .json::<model::dag::DagList>()
            .await
            .map(std::convert::Into::into)
            .map_err(std::convert::Into::into)
    }

    async fn toggle_dag(&self, dag_id: &str, is_paused: bool) -> Result<()> {
        self
            .base_api(Method::PATCH, &format!("dags/{dag_id}"))?
            .json(&serde_json::json!({"is_paused": !is_paused}))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    async fn get_dag_code(&self, dag: &crate::airflow::model::common::Dag) -> Result<String> {
        let r = self
            .base_api(Method::GET, &format!("dagSources/{}", dag.dag_id))?
            .build()?;
        let response = self.base.client.execute(r).await?.error_for_status()?;
        let dag_source: model::dag::DagSource = response.json().await?;
        Ok(dag_source.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airflow::{
        client::base::BaseClient,
        config::AirflowVersion,
        managed_services::conveyor::get_conveyor_environment_servers,
    };

    fn get_test_client() -> V2Client {
        let servers = get_conveyor_environment_servers().unwrap();
        let server = servers
            .into_iter()
            .find(|s| s.version == AirflowVersion::V3)
            .unwrap();
        let base = BaseClient::new(server).unwrap();
        V2Client::new(base)
    }

    #[tokio::test]
    // TODO: use a docker-compose Airflow v3 setup for testing instead
    async fn test_list_dags() {
        // Skip test if CONVEYOR_TESTS is not set to avoid panicking when credentials are unavailable
        if std::env::var("CONVEYOR_TESTS").unwrap_or_default() != "true" {
            eprintln!("Skipping test_list_dags: Set CONVEYOR_TESTS=true to run Conveyor integration tests");
            return;
        }

        let client = get_test_client();
        let daglist: DagList = client.list_dags().await.unwrap();
        assert!(!daglist.dags.is_empty());
    }

    #[tokio::test]
    async fn test_get_dag_code() {
        // Skip test if CONVEYOR_TESTS is not set to avoid panicking when credentials are unavailable
        if std::env::var("CONVEYOR_TESTS").unwrap_or_default() != "true" {
            eprintln!("Skipping test_get_dag_code: Set CONVEYOR_TESTS=true to run Conveyor integration tests");
            return;
        }

        let client = get_test_client();
        let daglist: DagList = client.list_dags().await.unwrap();
        let dag = &daglist.dags[0];
        let code = client.get_dag_code(dag).await.unwrap();
        assert!(code.contains(&dag.dag_id));
    }
}
