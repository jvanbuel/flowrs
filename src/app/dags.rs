use crate::app::error::Result;
use log::error;
use reqwest::{Method, Response};

use super::client::AirFlowClient;
use crate::model::dag::DagList;

impl AirFlowClient {
    pub async fn list_dags(&self) -> Result<DagList> {
        let r = self.base_api(Method::GET, "dags")?.build()?;
        let response = self.client.execute(r).await?;

        let daglist = response.json::<DagList>().await;
        match daglist {
            Ok(daglist) => Ok(daglist),
            Err(e) => {
                error!("Error parsing response: {}", e);
                Ok(DagList::default())
            }
        }
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
        let client = AirFlowClient::new(config.servers[0].clone()).unwrap();
        let daglist: DagList = client.list_dags().await.unwrap();
        assert_eq!(daglist.dags[0].dag_id, "dataset_consumes_1");
    }
}
