use log::{debug, info};
use reqwest::Method;

use super::model;
use super::V2Client;
use crate::client::read_json;
use crate::error::Result;

const PAGE_SIZE: usize = 50;

impl V2Client {
    pub async fn fetch_dags(&self) -> Result<model::dag::DagList> {
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
            let page: model::dag::DagList = read_json(response, "DAGs response").await?;

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

        Ok(model::dag::DagList {
            dags: all_dags,
            total_entries,
        })
    }

    pub async fn patch_dag_pause(&self, dag_id: &str, is_paused: bool) -> Result<()> {
        let request = self
            .base_api(Method::PATCH, &format!("dags/{dag_id}"))
            .await?
            .json(&serde_json::json!({"is_paused": !is_paused}));
        self.execute(request).await?;
        Ok(())
    }

    pub async fn fetch_dag_code(&self, dag_id: &str) -> Result<String> {
        let request = self
            .base_api(Method::GET, &format!("dagSources/{dag_id}"))
            .await?;
        let response = self.execute(request).await?;
        let dag_source: model::dag::DagSource = read_json(response, "DAG source response").await?;
        Ok(dag_source.content)
    }

    pub async fn fetch_dag_params(&self, dag_id: &str) -> Result<Option<serde_json::Value>> {
        // `params` lives on the details endpoint; the plain `dags/{dag_id}`
        // (DAGResponse) schema does not include it.
        let request = self
            .base_api(Method::GET, &format!("dags/{dag_id}/details"))
            .await?;
        let response = self.execute(request).await?;
        let body: serde_json::Value = read_json(response, "DAG details response").await?;
        Ok(body.get("params").cloned())
    }
}
