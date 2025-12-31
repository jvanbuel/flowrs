use anyhow::Result;
use async_trait::async_trait;
use reqwest::Method;

use super::model;
use super::V2Client;
use crate::airflow::{model::common::TaskList, traits::TaskOperations};

#[async_trait]
impl TaskOperations for V2Client {
    async fn list_tasks(&self, dag_id: &str) -> Result<TaskList> {
        let response = self
            .base_api(Method::GET, &format!("dags/{dag_id}/tasks"))
            .await?
            .send()
            .await?
            .error_for_status()?;

        let task_collection: model::task::TaskCollectionResponse = response.json().await?;
        Ok(task_collection.into())
    }
}
