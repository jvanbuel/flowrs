use anyhow::Result;
use async_trait::async_trait;
use reqwest::Method;

use crate::airflow::{model::common::DagStatsResponse, model::v1, traits::DagStatsOperations};

use super::V1Client;

#[async_trait]
impl DagStatsOperations for V1Client {
    /// Fetches statistics for the given DAG IDs from the v1 API.
    ///
    /// The request is sent to the `dagStats` endpoint with `dag_ids` joined by commas and the response
    /// is converted into the crate-level `DagStatsResponse`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn run_example() -> anyhow::Result<()> {
    /// let client = V1Client::new(/* ... */);
    /// let stats = client.get_dag_stats(vec!["example_dag"]).await?;
    /// let _ = stats; // use `stats` as needed
    /// # Ok(()) }
    /// ```
    async fn get_dag_stats(&self, dag_ids: Vec<&str>) -> Result<DagStatsResponse> {
        let response = self
            .base_api(Method::GET, "dagStats")?
            .query(&[("dag_ids", dag_ids.join(","))])
            .send()
            .await?;
        let dag_stats = response.json::<v1::dagstats::DagStatsResponse>().await?;
        Ok(dag_stats.into())
    }
}