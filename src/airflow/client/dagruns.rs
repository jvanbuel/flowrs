use log::debug;
use reqwest::{Method, Response};

use crate::airflow::model::dagrun::DagRunList;
use crate::app::error::Result;

use super::AirFlowClient;

impl AirFlowClient {
    pub async fn list_dagruns(&self, dag_id: &str) -> Result<DagRunList> {
        let response: Response = self
            .base_api(Method::GET, format!("dags/{dag_id}/dagRuns").as_str())
            .await?
            .query(&[("order_by", "-execution_date")])
            .send()
            .await?;
        let dagruns: DagRunList = response.json::<DagRunList>().await?;
        Ok(dagruns)
    }

    #[allow(dead_code)]
    pub async fn list_all_dagruns(&self) -> Result<DagRunList> {
        let response: Response = self
            .base_api(Method::POST, "dags/~/dagRuns/list")
            .await?
            .json(&serde_json::json!({"page_limit": 200}))
            .send()
            .await?;
        let dagruns: DagRunList = response.json::<DagRunList>().await?;
        Ok(dagruns)
    }

    pub async fn mark_dag_run(&self, dag_id: &str, dag_run_id: &str, status: &str) -> Result<()> {
        let _: Response = self
            .base_api(
                Method::PATCH,
                format!("dags/{dag_id}/dagRuns/{dag_run_id}").as_str(),
            )
            .await?
            .json(&serde_json::json!({"state": status}))
            .send()
            .await?;
        Ok(())
    }

    pub async fn clear_dagrun(&self, dag_id: &str, dag_run_id: &str) -> Result<()> {
        let _: Response = self
            .base_api(
                Method::POST,
                format!("dags/{dag_id}/dagRuns/{dag_run_id}/clear").as_str(),
            )
            .await?
            .json(&serde_json::json!({"dry_run": false}))
            .send()
            .await?;
        Ok(())
    }

    pub async fn trigger_dag_run(&self, dag_id: &str) -> Result<()> {
        let resp: Response = self
            .base_api(Method::POST, format!("dags/{dag_id}/dagRuns").as_str()).await?
            .json(&serde_json::json!({}))
            .send()
            .await?;

        debug!("{:?}", resp);
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::AirFlowClient;
    use super::DagRunList;
    use crate::airflow::config::FlowrsConfig;

    const TEST_CONFIG: &str = r#"[[servers]]
        name = "test"
        endpoint = "http://localhost:8080"

        [servers.auth.BasicAuth]
        username = "airflow"
        password = "airflow"
        "#;

    #[tokio::test]
    async fn test_list_dags() {
        let config: FlowrsConfig = toml::from_str(str::trim(TEST_CONFIG)).unwrap();
        let server = config.servers.unwrap()[0].clone();
        let client = AirFlowClient::new(server).unwrap();
        let first_dag = &client.list_dags().await.unwrap().dags[0];

        println!("{:?}", client.config);
        let dagrun_list = client.list_dagruns(first_dag.dag_id.as_str()).await;
        assert!(dagrun_list.is_ok());
    }

    #[tokio::test]
    async fn test_list_all_dags() {
        let config: FlowrsConfig = toml::from_str(str::trim(TEST_CONFIG)).unwrap();
        let server = config.servers.unwrap()[0].clone();
        let client = AirFlowClient::new(server).unwrap();

        let dagrun_list: DagRunList = client.list_all_dagruns().await.unwrap();
        assert!(!dagrun_list.dag_runs.is_empty());
    }
}
