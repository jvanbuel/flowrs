use std::error::Error;

use reqwest::{Method, Response};

use super::client::AirFlowClient;
use crate::model::dag::DagList;

impl AirFlowClient {
    pub async fn list_dags(&self) -> Result<DagList, Box<dyn Error + Send + Sync>> {
        let response: Response = self.base_api(Method::GET, "dags")?.send().await?;
        let daglist: DagList = response.json::<DagList>().await?;
        Ok(daglist)
    }

    pub async fn toggle_dag(
        &self,
        dag_id: &str,
        is_paused: bool,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
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

    use crate::app::auth::get_config;
    use crate::app::client::AirFlowClient;
    use crate::app::dags::DagList;

    #[tokio::test]
    async fn test_list_dags() {
        let binding = get_config(Some(Path::new(".flowrs")));
        let client = AirFlowClient::new(binding.unwrap().servers[1].clone());

        println!("{:?}", client.config);
        let daglist: DagList = client.list_dags().await.unwrap();
        assert_eq!(daglist.dags[0].dag_id, "dataset_consumes_1");
    }
}
