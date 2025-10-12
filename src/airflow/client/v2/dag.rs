use anyhow::Result;
use async_trait::async_trait;
use reqwest::Method;

use crate::airflow::{model::common::DagList, model::v2, traits::DagOperations};

use super::V2Client;

#[async_trait]
impl DagOperations for V2Client {
    /// Retrieves the list of DAGs from the Airflow v2 API.
    ///
    /// Returns the parsed `DagList` containing DAG metadata returned by the server.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn run_example() {
    /// // Initialize `client` appropriate for your environment.
    /// let client: V2Client = unimplemented!();
    /// let dag_list = client.list_dags().await.unwrap();
    /// // Use `dag_list` (e.g., inspect `dag_list.dags` or similar fields).
    /// # }
    /// ```
    async fn list_dags(&self) -> Result<DagList> {
        let r = self.base_api(Method::GET, "dags")?.build()?;
        let response = self.base.client.execute(r).await?;

        response
            .json::<v2::dag::DagList>()
            .await
            .map(|d| d.into())
            .map_err(|e| e.into())
    }

    /// Toggle a DAG's paused state.
    ///
    /// Sets the DAG's `is_paused` field to the opposite of the provided `is_paused` value by sending a PATCH request to the Airflow API.
    ///
    /// # Parameters
    ///
    /// - `dag_id`: Identifier of the DAG to update.
    /// - `is_paused`: The DAG's current paused state; the API will be set to `!is_paused`.
    ///
    /// # Returns
    ///
    /// `()` on success.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn run_example() -> anyhow::Result<()> {
    /// let client: crate::airflow::client::v2::V2Client = unimplemented!();
    /// // If the DAG is currently paused (true), this will unpause it (set to false).
    /// client.toggle_dag("example_dag", true).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn toggle_dag(&self, dag_id: &str, is_paused: bool) -> Result<()> {
        let _ = self
            .base_api(Method::PATCH, &format!("dags/{dag_id}"))?
            .json(&serde_json::json!({"is_paused": !is_paused}))
            .send()
            .await?;
        Ok(())
    }

    /// Fetches the source code for a DAG identified by its file token.
    ///
    /// # Returns
    ///
    /// `Ok(String)` containing the DAG source code on success, `Err` otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use anyhow::Result;
    /// # async fn example(client: &V2Client) -> Result<()> {
    /// let code = client.get_dag_code("my_file_token").await?;
    /// assert!(!code.is_empty());
    /// # Ok(()) }
    /// ```
    async fn get_dag_code(&self, file_token: &str) -> Result<String> {
        let r = self
            .base_api(Method::GET, &format!("dagSources/{file_token}"))?
            .build()?;
        let response = self.base.client.execute(r).await?;
        let code = response.text().await?;
        Ok(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airflow::{
        client::base::BaseClient,
        config::AirflowVersion,
        managed_services::conveyor::get_conveyor_environment_servers,
    };

    /// Constructs a V2Client configured for the test conveyor environment using the Airflow v3 server.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = get_test_client();
    /// // Use client in async tests, e.g. client.list_dags().await.unwrap();
    /// assert!(std::ptr::eq(&client as *const _, &client as *const _)); // trivial usage example
    /// ```
    fn get_test_client() -> V2Client {
        let servers = get_conveyor_environment_servers().unwrap();
        let server = servers
            .into_iter()
            .find(|s| s.version == AirflowVersion::V3)
            .unwrap();
        let base = BaseClient::new(server).unwrap();
        V2Client::new(base)
    }

    #[tokio::test]
    // TODO: use a docker-compose Airflow v3 setup for testing instead
    async fn test_list_dags() {
        let client = get_test_client();
        let daglist: DagList = client.list_dags().await.unwrap();
        assert!(!daglist.dags.is_empty());
    }
}