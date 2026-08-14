use reqwest::Method;

use super::model;
use super::V1Client;
use crate::client::read_json;
use crate::error::Result;

impl V1Client {
    pub async fn fetch_dag_stats(
        &self,
        dag_ids: Vec<&str>,
    ) -> Result<model::dagstats::DagStatsResponse> {
        let request = self
            .base_api(Method::GET, "dagStats")
            .await?
            .query(&[("dag_ids", dag_ids.join(","))]);
        let response = self.execute(request).await?;
        read_json(response, "DAG stats response").await
    }
}
