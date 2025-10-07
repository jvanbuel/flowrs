use anyhow::Result;
use async_trait::async_trait;
use reqwest::Method;

use crate::airflow::{model::common::DagStatsResponse, model::v1, traits::DagStatsOperations};

use super::V1Client;

#[async_trait]
impl DagStatsOperations for V1Client {
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
