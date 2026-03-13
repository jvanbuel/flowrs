use anyhow::Result;
use reqwest::Method;

use super::model;
use super::V2Client;

impl V2Client {
    pub async fn fetch_tasks(&self, dag_id: &str) -> Result<model::task::TaskCollectionResponse> {
        let response = self
            .base_api(Method::GET, &format!("dags/{dag_id}/tasks"))
            .await?
            .send()
            .await?
            .error_for_status()?;

        let task_collection: model::task::TaskCollectionResponse = response.json().await?;
        Ok(task_collection)
    }
}
