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

    pub async fn toggle_dag(
        &self,
        dag_id: &str,
        is_paused: bool,
    ) -> Result<()> {
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
    use std::path::Path;

    use crate::app::client::AirFlowClient;
    use crate::app::config::get_config;
    use crate::app::dags::DagList;

    #[tokio::test]
    async fn test_list_dags() {
        let binding = get_config(Some(Path::new(".flowrs"))).unwrap();
        let client = AirFlowClient::new(binding.servers[1].clone()).unwrap();

        println!("{:?}", client.config);
        let daglist: DagList = client.list_dags().await.unwrap();
        assert_eq!(daglist.dags[0].dag_id, "dataset_consumes_1");
    }
}
