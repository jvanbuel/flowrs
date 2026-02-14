use anyhow::Result;
use async_trait::async_trait;
use reqwest::Method;

use super::model;
use crate::airflow::{model::common::Log, traits::LogOperations};

use super::V1Client;

#[async_trait]
impl LogOperations for V1Client {
    async fn get_task_logs(
        &self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
        task_try: u32,
    ) -> Result<Log> {
        let response = self
            .base_api(
                Method::GET,
                &format!(
                    "dags/{dag_id}/dagRuns/{dag_run_id}/taskInstances/{task_id}/logs/{task_try}"
                ),
            )?
            .query(&[("full_content", "true")])
            .header("Accept", "application/json")
            .send()
            .await?
            .error_for_status()?;
        let log = response.json::<model::log::Log>().await?;
        Ok(log.into())
    }
}
