use anyhow::Result;
use async_trait::async_trait;
use reqwest::Method;

use crate::airflow::{model::common::Log, model::v1, traits::LogOperations};

use super::V1Client;

#[async_trait]
impl LogOperations for V1Client {
    /// Fetches the full log for a specific task try in a DAG run.
    ///
    /// Sends a GET request to the Airflow V1 API endpoint for the given dag, dag run, task, and try number and returns the parsed log.
    ///
    /// # Returns
    ///
    /// A `Log` containing the task's full log content.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// // `client` is an instance of `V1Client`
    /// let log = client.get_task_logs("example_dag", "run_id_123", "task_a", 1).await?;
    /// println!("{}", log); // inspect or process the retrieved log
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
        let log = response.json::<v1::log::Log>().await?;
        Ok(log.into())
    }
}