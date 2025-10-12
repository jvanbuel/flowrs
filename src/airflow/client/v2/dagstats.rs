use anyhow::Result;
use async_trait::async_trait;
use reqwest::Method;

use crate::airflow::{model::common::DagStatsResponse, model::v2, traits::DagStatsOperations};

use super::V2Client;

#[async_trait]
impl DagStatsOperations for V2Client {
    /// Fetches statistics for the specified DAGs from the V2 API.
    ///
    /// # Returns
    ///
    /// `DagStatsResponse` containing statistics for the requested DAGs.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn example(client: &V2Client) -> anyhow::Result<()> {
    /// let stats = client.get_dag_stats(vec!["sample-dbt", "sample-python-yaml"]).await?;
    /// assert_eq!(stats.dags.len(), 2);
    /// # Ok(())
    /// # }
    /// ```
    async fn get_dag_stats(&self, dag_ids: Vec<&str>) -> Result<DagStatsResponse> {
        let response = self
            .base_api(Method::GET, "dagStats")?
            .query(
                &dag_ids
                    .into_iter()
                    .map(|id| ("dag_ids", id))
                    .collect::<Vec<_>>(),
            )
            .send()
            .await?;
        let dag_stats = response.json::<v2::dagstats::DagStatsResponse>().await?;
        Ok(dag_stats.into())
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

    /// Constructs a V2Client configured for integration tests against a Conveyor-managed Airflow V3 server.
    ///
    /// The function selects the first available server with AirflowVersion::V3, creates a BaseClient for it, and returns a V2Client built from that base.
    ///
    /// # Examples
    ///
    /// ```
    /// let _client: V2Client = get_test_client();
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
    async fn test_dag_stats() {
        let client = get_test_client();
        let dag_stats = client
            .get_dag_stats(vec!["sample-dbt", "sample-python-yaml"])
            .await
            .unwrap();
        assert_eq!(dag_stats.dags.len(), 2);
    }
}