use anyhow::Result;
use async_trait::async_trait;
use log::{debug, info};
use reqwest::{Method, Response};

use crate::airflow::{model::common::TaskInstanceList, model::v2, traits::TaskInstanceOperations};

use super::V2Client;

#[async_trait]
impl TaskInstanceOperations for V2Client {
    /// Retrieves task instances for the specified DAG run.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // assuming `client` is a configured V2Client
    /// # async fn example(client: &V2Client) {
    /// let list = client.list_task_instances("example_dag", "example_run").await.unwrap();
    /// // use `list` which contains the task instances for the DAG run
    /// # }
    /// ```
    ///
    /// @returns `TaskInstanceList` containing the task instances for the given DAG run.
    async fn list_task_instances(
        &self,
        dag_id: &str,
        dag_run_id: &str,
    ) -> Result<TaskInstanceList> {
        let response: Response = self
            .base_api(
                Method::GET,
                &format!("dags/{dag_id}/dagRuns/{dag_run_id}/taskInstances"),
            )?
            .send()
            .await?;
        let daglist: v2::taskinstance::TaskInstanceList = response
            .json::<v2::taskinstance::TaskInstanceList>()
            .await?;
        info!("TaskInstances: {:?}", daglist);
        Ok(daglist.into())
    }

    /// Retrieves all task instances across all DAGs and DAG runs.
    ///
    /// # Returns
    ///
    /// `TaskInstanceList` containing every task instance visible to the client.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example(client: &crate::airflow::client::v2::V2Client) {
    /// let all = client.list_all_taskinstances().await.unwrap();
    /// assert!(all.items.len() >= 0);
    /// # }
    /// ```
    async fn list_all_taskinstances(&self) -> Result<TaskInstanceList> {
        let response: Response = self
            .base_api(Method::GET, "dags/~/dagRuns/~/taskInstances")?
            .send()
            .await?;
        let daglist: v2::taskinstance::TaskInstanceList = response
            .json::<v2::taskinstance::TaskInstanceList>()
            .await?;
        Ok(daglist.into())
    }

    /// Set the state of a task instance within a DAG run.
    ///
    /// Sends a PATCH request to the API to update the task instance identified by `dag_id`,
    /// `dag_run_id`, and `task_id` to the provided `status`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example(client: &impl crate::airflow::client::v2::TaskInstanceOperations) -> anyhow::Result<()> {
    /// client.mark_task_instance("example_dag", "run_2025_10_12", "task_1", "success").await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn mark_task_instance(
        &self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
        status: &str,
    ) -> Result<()> {
        let resp: Response = self
            .base_api(
                Method::PATCH,
                &format!("dags/{dag_id}/dagRuns/{dag_run_id}/taskInstances/{task_id}"),
            )?
            .json(&serde_json::json!({"new_state": status, "dry_run": false}))
            .send()
            .await?;
        debug!("{:?}", resp);
        Ok(())
    }

    /// Clears the specified task instance for a DAG run, resetting its state and also clearing downstream tasks and related DAG run state.
    ///
    /// This sends a request to the Airflow API to clear (reset) the given task instance identified by `dag_id`, `dag_run_id`, and `task_id`. The operation includes downstream tasks and resets DAG run state.
    ///
    /// # Parameters
    ///
    /// - `dag_id`: Identifier of the DAG containing the task.
    /// - `dag_run_id`: Identifier of the DAG run containing the task instance to clear.
    /// - `task_id`: Identifier of the task to clear.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use anyhow::Result;
    /// # async fn example() -> Result<()> {
    /// let client = V2Client::new(/* ... */);
    /// client.clear_task_instance("example_dag", "manual__2025-10-01T00:00:00+00:00", "my_task").await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn clear_task_instance(
        &self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
    ) -> Result<()> {
        let resp: Response = self
            .base_api(Method::POST, &format!("dags/{dag_id}/clearTaskInstances"))?
            .json(&serde_json::json!(
                {
                    "dry_run": false,
                    "task_ids": [task_id],
                    "dag_run_id": dag_run_id,
                    "include_downstream": true,
                    "only_failed": false,
                    "reset_dag_runs": true,
                }
            ))
            .send()
            .await?;
        debug!("{:?}", resp);
        Ok(())
    }
}