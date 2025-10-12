use anyhow::Result;
use async_trait::async_trait;
use log::{debug, info};
use reqwest::{Method, Response};

use crate::airflow::{model::common::TaskInstanceList, model::v1, traits::TaskInstanceOperations};

use super::V1Client;

#[async_trait]
impl TaskInstanceOperations for V1Client {
    /// Fetches the task instances for a specific DAG run.
    ///
    /// Returns a `TaskInstanceList` containing the task instances for the given `dag_id` and `dag_run_id`,
    /// or an error if the request or response deserialization fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn run_example(client: &V1Client) -> anyhow::Result<()> {
    /// let list = client.list_task_instances("example_dag", "example_run").await?;
    /// // use `list` as needed
    /// # Ok(())
    /// # }
    /// ```
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
        let daglist: v1::taskinstance::TaskInstanceCollectionResponse = response
            .json::<v1::taskinstance::TaskInstanceCollectionResponse>()
            .await?;
        info!("TaskInstances: {:?}", daglist);
        Ok(daglist.into())
    }

    /// Retrieves all task instances across all DAGs and DAG runs.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use crate::airflow::client::v1::V1Client;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let client = V1Client::new(/* config */);
    /// let task_instances = client.list_all_taskinstances().await?;
    /// // inspect or use `task_instances`
    /// # Ok(())
    /// # }
    /// ```
    async fn list_all_taskinstances(&self) -> Result<TaskInstanceList> {
        let response: Response = self
            .base_api(Method::GET, "dags/~/dagRuns/~/taskInstances")?
            .send()
            .await?;

        println!("Response: {:?}", response);
        let daglist: v1::taskinstance::TaskInstanceCollectionResponse = response
            .json::<v1::taskinstance::TaskInstanceCollectionResponse>()
            .await?;
        Ok(daglist.into())
    }

    /// Updates the state of a task instance for a specific DAG run.
    ///
    /// Sends a PATCH request to Airflow to set the task instance's state to `status`. Returns `Ok(())` on success; errors are propagated.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use crate::airflow::client::v1::V1Client;
    /// # async fn example(client: &V1Client) -> anyhow::Result<()> {
    /// client.mark_task_instance("my_dag", "my_dag_run", "my_task", "success").await?;
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

    /// Clears a specific task instance for a DAG run in Airflow by calling the DAG's clearTaskInstances endpoint.
    ///
    /// This requests the Airflow API to clear the specified task (including downstream tasks) for the given DAG run.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # async fn _example() -> anyhow::Result<()> {
    /// // Construct a V1Client configured for your Airflow instance:
    /// let client = V1Client::new(/* base_url, auth, etc. */)?;
    /// // Clear the task instance
    /// client.clear_task_instance("example_dag", "example_run_id", "task_1").await?;
    /// # Ok(()) }
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