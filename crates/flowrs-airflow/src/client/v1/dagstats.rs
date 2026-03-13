use anyhow::Result;
use reqwest::Method;

use super::model;
use super::V1Client;

impl V1Client {
    pub async fn fetch_dag_stats(&self, dag_ids: Vec<&str>) -> Result<model::dagstats::DagStatsResponse> {
        let response = self
            .base_api(Method::GET, "dagStats")
            .await?
            .query(&[("dag_ids", dag_ids.join(","))])
            .send()
            .await?
            .error_for_status()?;

        let dag_stats = response.json::<model::dagstats::DagStatsResponse>().await?;
        Ok(dag_stats)
    }
}
