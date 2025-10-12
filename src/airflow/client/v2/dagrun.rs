use anyhow::Result;
use async_trait::async_trait;
use log::debug;
use reqwest::{Method, Response};

use crate::airflow::{model::common::DagRunList, model::v2, traits::DagRunOperations};

use super::V2Client;

#[async_trait]
impl DagRunOperations for V2Client {
    /// Fetches the DAG runs for the given DAG, ordered by `logical_date` descending.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn run_example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = V2Client::default();
    /// let result = client.list_dagruns("example_dag").await?;
    /// assert!(result.dag_runs.is_empty() || result.dag_runs.len() > 0);
    /// # Ok(()) }
    /// ```
    —
    async fn list_dagruns(&self, dag_id: &str) -> Result<DagRunList> {
        let response: Response = self
            .base_api(Method::GET, &format!("dags/{dag_id}/dagRuns"))?
            .query(&[("order_by", "-logical_date")])
            .send()
            .await?;
        let dagruns: v2::dagrun::DagRunList = response.json::<v2::dagrun::DagRunList>().await?;
        Ok(dagruns.into())
    }

    /// Fetches DAG runs across all DAGs by calling the v2 list endpoint.
    ///
    /// Sends a request to the DAG runs listing endpoint with a page limit of 200 and returns the parsed `DagRunList`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = /* construct V2Client */ ;
    /// let dagruns = client.list_all_dagruns().await?;
    /// // use `dagruns`
    /// # Ok(())
    /// # }
    /// ```
    async fn list_all_dagruns(&self) -> Result<DagRunList> {
        let response: Response = self
            .base_api(Method::POST, "dags/~/dagRuns/list")?
            .json(&serde_json::json!({"page_limit": 200}))
            .send()
            .await?;
        let dagruns: v2::dagrun::DagRunList = response.json::<v2::dagrun::DagRunList>().await?;
        Ok(dagruns.into())
    }

    /// Marks the state of a DAG run identified by the given DAG ID and DAG run ID.
    ///
    /// Sends a PATCH request updating the run's `state` to `status`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn doc_example(client: &crate::airflow::client::v2::V2Client) -> anyhow::Result<()> {
    /// client.mark_dag_run("example_dag", "run_1", "success").await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, error propagated otherwise.
    async fn mark_dag_run(&self, dag_id: &str, dag_run_id: &str, status: &str) -> Result<()> {
        let _: Response = self
            .base_api(
                Method::PATCH,
                &format!("dags/{dag_id}/dagRuns/{dag_run_id}"),
            )?
            .json(&serde_json::json!({"state": status}))
            .send()
            .await?;
        Ok(())
    }

    /// Clears all task instances for the specified DAG run on the Airflow server.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use futures::executor::block_on;
    /// // `client` is a configured `V2Client`.
    /// // This will send a clear request for the given DAG run and return on success.
    /// # let client = /* create or obtain a V2Client */ unimplemented!();
    /// block_on(async { client.clear_dagrun("example_dag", "example_run_id").await.unwrap() });
    /// ```
    async fn clear_dagrun(&self, dag_id: &str, dag_run_id: &str) -> Result<()> {
        let _: Response = self
            .base_api(
                Method::POST,
                &format!("dags/{dag_id}/dagRuns/{dag_run_id}/clear"),
            )?
            .json(&serde_json::json!({"dry_run": false}))
            .send()
            .await?;
        Ok(())
    }

    /// Triggers a new DAG run for the specified DAG.
    ///
    /// Returns `Ok(())` if the DAG run was successfully created, `Err` otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = V2Client::new(/* ... */);
    /// client.trigger_dag_run("example_dag").await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn trigger_dag_run(&self, dag_id: &str) -> Result<()> {
        let resp: Response = self
            .base_api(Method::POST, &format!("dags/{dag_id}/dagRuns"))?
            .json(&serde_json::json!({}))
            .send()
            .await?;
        debug!("{:?}", resp);
        Ok(())
    }
}