use anyhow::Result;
use async_trait::async_trait;
use reqwest::Method;

use super::model;
use crate::airflow::{model::common::DagStatsResponse, traits::DagStatsOperations};

use super::V1Client;

#[async_trait]
impl DagStatsOperations for V1Client {
    async fn get_dag_stats(&self, dag_ids: Vec<&str>) -> Result<DagStatsResponse> {
        let response = self
            .base_api(Method::GET, "dagStats")
            .await?
            .query(&[("dag_ids", dag_ids.join(","))])
            .send()
            .await?
            .error_for_status()?;

        let dag_stats = response.json::<model::dagstats::DagStatsResponse>().await?;
        Ok(dag_stats.into())
    }
}
