use anyhow::Result;
use async_trait::async_trait;
use log::debug;
use reqwest::{Method, Response};

use crate::airflow::{model::common::DagRunList, model::v1, traits::DagRunOperations};

use super::V1Client;

#[async_trait]
impl DagRunOperations for V1Client {
    /// Lists DAG runs for the specified DAG, ordered by execution date (newest first).
    ///
    /// Retrieves DAG runs for `dag_id` from the Airflow v1 API and converts the response into a `DagRunList`.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn example(client: &crate::V1Client) -> anyhow::Result<()> {
    /// let runs = client.list_dagruns("example_dag").await?;
    /// assert!(!runs.dag_runs.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Returns
    /// `DagRunList` containing DAG runs for the specified DAG, ordered by execution date (newest first).
    async fn list_dagruns(&self, dag_id: &str) -> Result<DagRunList> {
        let response: Response = self
            .base_api(Method::GET, &format!("dags/{dag_id}/dagRuns"))?
            .query(&[("order_by", "-execution_date")])
            .send()
            .await?;

        let dagruns: v1::dagrun::DAGRunCollectionResponse = response
            .json::<v1::dagrun::DAGRunCollectionResponse>()
            .await?;
        Ok(dagruns.into())
    }

    /// Lists DAG runs across all DAGs and returns them as a `DagRunList`.
    ///
    /// Posts a request to the Airflow endpoint `dags/~/dagRuns/list` with a page limit of 200 and parses the response into a `DagRunList`.
    ///
    /// # Returns
    ///
    /// A `DagRunList` containing the retrieved DAG runs.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // assuming `client` is a configured `V1Client`
    /// let list = client.list_all_dagruns().await.unwrap();
    /// assert!(!list.dag_runs.is_empty());
    /// ```
    async fn list_all_dagruns(&self) -> Result<DagRunList> {
        let response: Response = self
            .base_api(Method::POST, "dags/~/dagRuns/list")?
            .json(&serde_json::json!({"page_limit": 200}))
            .send()
            .await?;
        let dagruns: v1::dagrun::DAGRunCollectionResponse = response
            .json::<v1::dagrun::DAGRunCollectionResponse>()
            .await?;
        Ok(dagruns.into())
    }

    /// Sets the state of a specific DAG run.
    ///
    /// # Parameters
    ///
    /// - `dag_id`: The identifier of the DAG containing the run.
    /// - `dag_run_id`: The identifier of the DAG run to update.
    /// - `status`: The target run state (for example `"success"`, `"failed"`, or `"running"`).
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn example_usage(client: &impl crate::airflow::DagRunOperations) -> anyhow::Result<()> {
    /// client.mark_dag_run("example_dag", "manual__2025-10-12T00:00:00+00:00", "success").await?;
    /// # Ok(())
    /// # }
    /// ```
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

    /// Clears all task instances for a specific DAG run on the Airflow server.
    ///
    /// Sends a clear request for the DAG run identified by `dag_id` and `dag_run_id` (not a dry run).
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn example(client: &crate::V1Client) {
    /// client.clear_dagrun("example_dag", "example_run_id").await.unwrap();
    /// # }
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

    /// Triggers a new DAG run for the specified DAG ID.
    ///
    /// Sends a POST request to the DAG runs endpoint to create a new run.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, an error otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn example(client: &crate::V1Client) -> anyhow::Result<()> {
    /// client.trigger_dag_run("example_dag_id").await?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airflow::client::base::BaseClient;

    const TEST_CONFIG: &str = r#"[[servers]]
        name = "test"
        endpoint = "http://localhost:8080"

        [servers.auth.Basic]
        username = "airflow"
        password = "airflow"
        "#;

    /// Creates a `V1Client` configured from the embedded `TEST_CONFIG` for use in tests.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let client = get_test_client();
    /// // use `client` in test helpers
    /// ```
    fn get_test_client() -> V1Client {
        let config: crate::airflow::config::FlowrsConfig =
            toml::from_str(TEST_CONFIG.trim()).unwrap();
        let base = BaseClient::new(config.servers.unwrap()[0].clone()).unwrap();
        V1Client::new(base)
    }

    #[tokio::test]
    async fn test_list_dagruns() {
        let client = get_test_client();
        let dagruns = client.list_dagruns("example_dag_decorator").await.unwrap();
        assert!(!dagruns.dag_runs.is_empty());
    }
}