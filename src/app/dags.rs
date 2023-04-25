use std::{error::Error};

use reqwest::{Method, Response};
use serde::{Deserialize, Serialize};

use super::client::AirFlowClient;
use crate::model::dag::Dag;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DagList {
    pub dags: Vec<Dag>,
    pub total_entries: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToggleUpdate {
    pub is_paused: bool,
}

impl AirFlowClient<'_> {
    pub async fn list_dags(&self) -> Result<DagList, Box<dyn Error>> {
        let response: Response = self.base_api(Method::GET, "dags")?.send().await?;
        let daglist: DagList = response.json::<DagList>().await?;
        Ok(daglist)
    }

    pub async fn toggle_dag(&self, dag_id: &str, is_paused: bool) -> Result<(), Box<dyn Error>> {

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
        let binding = get_config(Some(&Path::new(".flowrs")));
        let client = AirFlowClient::new(&binding.servers[1]);

        println!("{:?}", client.config);
        let daglist: DagList = client.list_dags().await.unwrap();
        assert_eq!(daglist.dags[0].dag_id, "dataset_consumes_1");
    }
}
