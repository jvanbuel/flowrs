use anyhow::Result;
use log::debug;
use reqwest::Method;

use super::model;
use super::V2Client;

impl V2Client {
    pub async fn fetch_task_logs(
        &self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
        task_try: u32,
    ) -> Result<model::log::Log> {
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
        Ok(log)
    }
}
