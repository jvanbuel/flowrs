use anyhow::Result;
use async_trait::async_trait;
use reqwest::Method;

use crate::airflow::{model::common::Log, model::v2, traits::LogOperations};

use super::V2Client;

#[async_trait]
impl LogOperations for V2Client {
    async fn get_task_logs(
        &self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
        task_try: u16,
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
            .await?;
        let log = response.json::<v2::log::Log>().await?;
        Ok(log.into())
    }
}
