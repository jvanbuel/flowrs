use anyhow::Result;
use async_trait::async_trait;
use reqwest::Method;

use crate::airflow::{model::common::Log, model::v2, traits::LogOperations};

use super::V2Client;

#[async_trait]
impl LogOperations for V2Client {
    /// Fetches the full log for a specific task instance try from the Airflow v2 API.
    ///
    /// Returns the task instance log converted to the crate's `Log` type.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// // `client` is an initialized `V2Client`
    /// let client = /* V2Client::new(...) */ unimplemented!();
    /// let log = client.get_task_logs("example_dag", "example_run", "example_task", 1).await?;
    /// // use `log`
    /// # Ok(())
    /// # }
    /// ```
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