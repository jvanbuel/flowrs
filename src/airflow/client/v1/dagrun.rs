use anyhow::Result;
use async_trait::async_trait;
use log::debug;
use reqwest::{Method, Response};

use crate::airflow::{model::common::DagRunList, model::v1, traits::DagRunOperations};

use super::V1Client;

#[async_trait]
impl DagRunOperations for V1Client {
    async fn list_dagruns(&self, dag_id: &str) -> Result<DagRunList> {
        let response: Response = self
            .base_api(Method::GET, &format!("dags/{dag_id}/dagRuns"))?
            .query(&[("order_by", "-execution_date")])
            .send()
            .await?;

        let dagruns: v1::dagrun::DAGRunCollectionResponse = response
            .json::<v1::dagrun::DAGRunCollectionResponse>()
            .await?;
        Ok(dagruns.into())
    }

    async fn list_all_dagruns(&self) -> Result<DagRunList> {
        let response: Response = self
            .base_api(Method::POST, "dags/~/dagRuns/list")?
            .json(&serde_json::json!({"page_limit": 200}))
            .send()
            .await?;
        let dagruns: v1::dagrun::DAGRunCollectionResponse = response
            .json::<v1::dagrun::DAGRunCollectionResponse>()
            .await?;
        Ok(dagruns.into())
    }

    async fn mark_dag_run(&self, dag_id: &str, dag_run_id: &str, status: &str) -> Result<()> {
        let _: Response = self
            .base_api(
                Method::PATCH,
                &format!("dags/{dag_id}/dagRuns/{dag_run_id}"),
            )?
            .json(&serde_json::json!({"state": status}))
            .send()
            .await?;
        Ok(())
    }

    async fn clear_dagrun(&self, dag_id: &str, dag_run_id: &str) -> Result<()> {
        let _: Response = self
            .base_api(
                Method::POST,
                &format!("dags/{dag_id}/dagRuns/{dag_run_id}/clear"),
            )?
            .json(&serde_json::json!({"dry_run": false}))
            .send()
            .await?;
        Ok(())
    }

    async fn trigger_dag_run(&self, dag_id: &str) -> Result<()> {
        let resp: Response = self
            .base_api(Method::POST, &format!("dags/{dag_id}/dagRuns"))?
            .json(&serde_json::json!({}))
            .send()
            .await?;
        debug!("{:?}", resp);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airflow::client::base::BaseClient;

    const TEST_CONFIG: &str = r#"[[servers]]
        name = "test"
        endpoint = "http://localhost:8080"

        [servers.auth.Basic]
        username = "airflow"
        password = "airflow"
        "#;

    fn get_test_client() -> V1Client {
        let config: crate::airflow::config::FlowrsConfig =
            toml::from_str(TEST_CONFIG.trim()).unwrap();
        let base = BaseClient::new(config.servers.unwrap()[0].clone()).unwrap();
        V1Client::new(base)
    }

    #[tokio::test]
    async fn test_list_dagruns() {
        let client = get_test_client();
        let dagruns = client.list_dagruns("example_dag_decorator").await.unwrap();
        assert!(!dagruns.dag_runs.is_empty());
    }
}
