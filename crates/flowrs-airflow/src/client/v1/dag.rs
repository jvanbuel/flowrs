use log::{debug, info};
use reqwest::Method;

use super::model::dag::DagCollectionResponse;
use super::V1Client;
use crate::client::read_json;
use crate::error::Result;

const PAGE_SIZE: usize = 50;

impl V1Client {
    pub async fn fetch_dags(&self) -> Result<DagCollectionResponse> {
        let mut all_dags = Vec::new();
        let mut offset = 0;
        let limit = PAGE_SIZE;
        let mut total_entries;

        loop {
            let request = self
                .base_api(Method::GET, "dags")
                .await?
                .query(&[("limit", limit.to_string()), ("offset", offset.to_string())]);
            let response = self.execute(request).await?;
            let page: DagCollectionResponse = read_json(response, "DAGs response").await?;

            total_entries = page.total_entries;
            let fetched_count = page.dags.len();
            all_dags.extend(page.dags);

            debug!("Fetched {fetched_count} DAGs, offset: {offset}, total: {total_entries}");

            let total_usize = usize::try_from(total_entries).unwrap_or(usize::MAX);
            if fetched_count < limit || all_dags.len() >= total_usize {
                break;
            }

            offset += fetched_count;
        }

        info!(
            "Fetched total {} DAGs out of {}",
            all_dags.len(),
            total_entries
        );

        Ok(DagCollectionResponse {
            dags: all_dags,
            total_entries,
        })
    }

    pub async fn patch_dag_pause(&self, dag_id: &str, is_paused: bool) -> Result<()> {
        let request = self
            .base_api(Method::PATCH, &format!("dags/{dag_id}"))
            .await?
            .query(&[("update_mask", "is_paused")])
            .json(&serde_json::json!({"is_paused": !is_paused}));
        self.execute(request).await?;
        Ok(())
    }

    pub async fn fetch_dag_code(&self, file_token: &str) -> Result<String> {
        let request = self
            .base_api(Method::GET, &format!("dagSources/{file_token}"))
            .await?;
        let response = self.execute(request).await?;
        let code = response.text().await?;
        Ok(code)
    }

    pub async fn fetch_dag_params(&self, dag_id: &str) -> Result<Option<serde_json::Value>> {
        let request = self
            .base_api(Method::GET, &format!("dags/{dag_id}/details"))
            .await?;
        let response = self.execute(request).await?;
        let body: serde_json::Value = read_json(response, "DAG details response").await?;
        Ok(body.get("params").cloned())
    }
}
