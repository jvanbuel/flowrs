use reqwest::Method;

use super::model;
use super::V2Client;
use crate::client::read_json;
use crate::error::Result;

impl V2Client {
    pub async fn fetch_tasks(&self, dag_id: &str) -> Result<model::task::TaskCollectionResponse> {
        let request = self
            .base_api(Method::GET, &format!("dags/{dag_id}/tasks"))
            .await?;
        let response = self.execute(request).await?;
        read_json(response, "tasks response").await
    }
}
