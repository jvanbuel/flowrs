use log::debug;
use reqwest::Method;

use super::model;
use super::V2Client;
use crate::client::read_json;
use crate::error::Result;

impl V2Client {
    pub async fn fetch_task_logs(
        &self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
        task_try: u32,
    ) -> Result<model::log::Log> {
        let request = self
            .base_api(
                Method::GET,
                &format!(
                    "dags/{dag_id}/dagRuns/{dag_run_id}/taskInstances/{task_id}/logs/{task_try}"
                ),
            )
            .await?
            .query(&[("full_content", "true")])
            .header("Accept", "application/json");
        let response = self.execute(request).await?;

        debug!("Response: {response:?}");
        let log: model::log::Log = read_json(response, "task logs response").await?;
        debug!("Parsed Log: {log:?}");
        Ok(log)
    }
}
