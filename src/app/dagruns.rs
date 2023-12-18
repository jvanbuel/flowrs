use crate::app::error::Result;
use reqwest::{Method, Response};

use crate::model::dagrun::DagRunList;

use super::client::AirFlowClient;

impl AirFlowClient {
    pub async fn list_dagruns(&self, dag_id: &str) -> Result<DagRunList> {
        let response: Response = self
            .base_api(Method::GET, format!("dags/{dag_id}/dagRuns").as_str())?
            .query(&[("order_by", "-execution_date")])
            .send()
            .await?;
        let dagruns: DagRunList = response.json::<DagRunList>().await?;
        Ok(dagruns)
    }

    pub async fn list_all_dagruns(&self) -> Result<DagRunList> {
        let response: Response = self
            .base_api(Method::POST, "dags/~/dagRuns/list")?
            .json(&serde_json::json!({"page_limit": 200}))
            .send()
            .await?;
        let dagruns: DagRunList = response.json::<DagRunList>().await?;
        Ok(dagruns)
    }

    pub async fn clear_dagrun(&self, dag_id: &str, dag_run_id: &str) -> Result<()> {
        let _: Response = self
            .base_api(
                Method::POST,
                format!("dags/{dag_id}/dagRuns/{dag_run_id}/clear").as_str(),
            )?
            .send()
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::app::client::AirFlowClient;
    use crate::app::config::FlowrsConfig;
    use crate::app::dagruns::DagRunList;

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
        let server = config.servers[0].clone();
        let client = AirFlowClient::new(server).unwrap();
        let first_dag = &client.list_dags().await.unwrap().dags[0];

        println!("{:?}", client.config);
        let dagrun_list: DagRunList = client
            .list_dagruns(first_dag.dag_id.as_str())
            .await
            .unwrap();
        assert!(!dagrun_list.dag_runs.is_empty());
    }

    #[tokio::test]
    async fn test_list_all_dags() {
        let config: FlowrsConfig = toml::from_str(str::trim(TEST_CONFIG)).unwrap();
        let server = config.servers[0].clone();
        let client = AirFlowClient::new(server).unwrap();

        let dagrun_list: DagRunList = client.list_all_dagruns().await.unwrap();
        assert!(!dagrun_list.dag_runs.is_empty());
    }
}
