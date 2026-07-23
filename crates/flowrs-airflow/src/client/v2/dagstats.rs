use reqwest::Method;

use super::model;
use super::V2Client;
use crate::client::read_json;
use crate::error::Result;

impl V2Client {
    pub async fn fetch_dag_stats(
        &self,
        dag_ids: Vec<&str>,
    ) -> Result<model::dagstats::DagStatsResponse> {
        let request = self.base_api(Method::GET, "dagStats").await?.query(
            &dag_ids
                .into_iter()
                .map(|id| ("dag_ids", id))
                .collect::<Vec<_>>(),
        );
        let response = self.execute(request).await?;
        read_json(response, "DAG stats response").await
    }
}
