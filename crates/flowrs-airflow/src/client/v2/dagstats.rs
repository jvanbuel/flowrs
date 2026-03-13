use anyhow::Result;
use reqwest::Method;

use super::model;
use super::V2Client;

impl V2Client {
    pub async fn fetch_dag_stats(
        &self,
        dag_ids: Vec<&str>,
    ) -> Result<model::dagstats::DagStatsResponse> {
        let response = self
            .base_api(Method::GET, "dagStats")
            .await?
            .query(
                &dag_ids
                    .into_iter()
                    .map(|id| ("dag_ids", id))
                    .collect::<Vec<_>>(),
            )
            .send()
            .await?
            .error_for_status()?;
        let dag_stats = response.json::<model::dagstats::DagStatsResponse>().await?;
        Ok(dag_stats)
    }
}
