use anyhow::Result;
use async_trait::async_trait;
use log::debug;
use reqwest::Method;

use super::model;
use flowrs_airflow_model::model::common::Log;
use flowrs_airflow_model::traits::LogOperations;

use super::V2Client;

#[async_trait]
impl LogOperations for V2Client {
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
            )
            .await?
            .query(&[("full_content", "true")])
            .header("Accept", "application/json")
            .send()
            .await?
            .error_for_status()?;

        debug!("Response: {response:?}");
        let log = response.json::<model::log::Log>().await?;
        debug!("Parsed Log: {log:?}");
        Ok(log.into())
    }
}

// From trait implementation for v2 log model
impl From<model::log::Log> for Log {
    fn from(value: model::log::Log) -> Self {
        Self {
            continuation_token: value.continuation_token,
            content: value.content.to_string(),
        }
    }
}
