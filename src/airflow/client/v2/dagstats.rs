use anyhow::Result;
use async_trait::async_trait;
use reqwest::Method;

use super::model;
use crate::airflow::{model::common::DagStatsResponse, traits::DagStatsOperations};

use super::V2Client;

#[async_trait]
impl DagStatsOperations for V2Client {
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
            .await?
            .error_for_status()?;
        let dag_stats = response.json::<model::dagstats::DagStatsResponse>().await?;
        Ok(dag_stats.into())
    }
}
