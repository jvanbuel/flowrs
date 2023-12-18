use crate::app::error::Result;
use reqwest::{Method, Response};

use super::client::AirFlowClient;
use crate::model::dag::DagList;

impl AirFlowClient {
    pub async fn list_dags(&self) -> Result<DagList> {
        let response: Response = self.base_api(Method::GET, "dags")?.send().await?;
        let daglist: DagList = response.json::<DagList>().await?;
        Ok(daglist)
    }

    pub async fn toggle_dag(&self, dag_id: &str, is_paused: bool) -> Result<()> {
        let _: Response = self
            .base_api(Method::PATCH, &format!("dags/{dag_id}"))?
            .query(&[("update_mask", "is_paused")])
            .json(&serde_json::json!({"is_paused": !is_paused}))
            .send()
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::app::client::AirFlowClient;
    use crate::app::config::FlowrsConfig;
    use crate::app::dags::DagList;

    #[tokio::test]
    async fn test_list_dags() {
        let configuration = r#"[[servers]]
        name = "test"
        endpoint = "http://localhost:8080"

        [servers.auth.BasicAuth]
        username = "airflow"
        password = "airflow"
        "#;

        let config: FlowrsConfig = toml::from_str(str::trim(configuration)).unwrap();
        let client = AirFlowClient::new(config.servers[0].clone()).unwrap();
        let daglist: DagList = client.list_dags().await.unwrap();
        assert_eq!(daglist.dags[0].dag_id, "dataset_consumes_1");
    }
}
