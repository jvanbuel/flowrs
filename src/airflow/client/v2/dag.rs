use anyhow::Result;
use async_trait::async_trait;
use reqwest::Method;

use crate::airflow::{model::common::DagList, model::v2, traits::DagOperations};

use super::V2Client;

#[async_trait]
impl DagOperations for V2Client {
    async fn list_dags(&self) -> Result<DagList> {
        let r = self.base_api(Method::GET, "dags")?.build()?;
        let response = self.base.client.execute(r).await?;

        response
            .json::<v2::dag::DagList>()
            .await
            .map(|d| d.into())
            .map_err(|e| e.into())
    }

    async fn toggle_dag(&self, dag_id: &str, is_paused: bool) -> Result<()> {
        let _ = self
            .base_api(Method::PATCH, &format!("dags/{dag_id}"))?
            .json(&serde_json::json!({"is_paused": !is_paused}))
            .send()
            .await?;
        Ok(())
    }

    async fn get_dag_code(&self, file_token: &str) -> Result<String> {
        let r = self
            .base_api(Method::GET, &format!("dagSources/{file_token}"))?
            .build()?;
        let response = self.base.client.execute(r).await?;
        let code = response.text().await?;
        Ok(code)
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
        let client = get_test_client();
        let daglist: DagList = client.list_dags().await.unwrap();
        assert!(!daglist.dags.is_empty());
    }
}
