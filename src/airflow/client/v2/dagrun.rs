use anyhow::Result;
use async_trait::async_trait;
use log::debug;
use reqwest::{Method, Response};

use crate::airflow::{model::common::DagRunList, traits::DagRunOperations};
use super::model;

use super::V2Client;

#[async_trait]
impl DagRunOperations for V2Client {
    async fn list_dagruns(&self, dag_id: &str) -> Result<DagRunList> {
        let response: Response = self
            .base_api(Method::GET, &format!("dags/{dag_id}/dagRuns"))?
            .query(&[("order_by", "-start_date"), ("limit", "50")])
            .send()
            .await?
            .error_for_status()?;
        let dagruns: model::dagrun::DagRunList = response.json::<model::dagrun::DagRunList>().await?;
        Ok(dagruns.into())
    }

    async fn list_all_dagruns(&self) -> Result<DagRunList> {
        let response: Response = self
            .base_api(Method::POST, "dags/~/dagRuns/list")?
            .json(&serde_json::json!({"page_limit": 200}))
            .send()
            .await?
            .error_for_status()?;
        let dagruns: model::dagrun::DagRunList = response.json::<model::dagrun::DagRunList>().await?;
        Ok(dagruns.into())
    }

    async fn mark_dag_run(&self, dag_id: &str, dag_run_id: &str, status: &str) -> Result<()> {
        self
            .base_api(
                Method::PATCH,
                &format!("dags/{dag_id}/dagRuns/{dag_run_id}"),
            )?
            .json(&serde_json::json!({"state": status}))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    async fn clear_dagrun(&self, dag_id: &str, dag_run_id: &str) -> Result<()> {
        self
            .base_api(
                Method::POST,
                &format!("dags/{dag_id}/dagRuns/{dag_run_id}/clear"),
            )?
            .json(&serde_json::json!({"dry_run": false}))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    async fn trigger_dag_run(&self, dag_id: &str, logical_date: Option<&str>) -> Result<()> {
        let body = serde_json::json!({"logical_date": logical_date});

        let resp: Response = self
            .base_api(Method::POST, &format!("dags/{dag_id}/dagRuns"))?
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        debug!("{resp:?}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airflow::{
        client::base::BaseClient,
        config::AirflowVersion,
        managed_services::conveyor::get_conveyor_environment_servers,
        traits::DagOperations,
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
    async fn test_list_dagruns() {
        // Skip test if CONVEYOR_TESTS is not set to avoid panicking when credentials are unavailable
        if std::env::var("CONVEYOR_TESTS").unwrap_or_default() != "true" {
            eprintln!("Skipping test_list_dagruns: Set CONVEYOR_TESTS=true to run Conveyor integration tests");
            return;
        }

        let client = get_test_client();
        let dags = client.list_dags().await.unwrap();
        let dag_id = &dags.dags[0].dag_id;
        let dagruns = client.list_dagruns(dag_id).await.unwrap();
        assert!(!dagruns.dag_runs.is_empty());
    }
}
