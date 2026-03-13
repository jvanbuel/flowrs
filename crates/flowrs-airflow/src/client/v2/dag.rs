use anyhow::Result;
use log::{debug, info};
use reqwest::Method;

use super::model;
use super::V2Client;

const PAGE_SIZE: usize = 50;

impl V2Client {
    pub async fn fetch_dags(&self) -> Result<model::dag::DagList> {
        let mut all_dags = Vec::new();
        let mut offset = 0;
        let limit = PAGE_SIZE;
        let mut total_entries;

        loop {
            let response = self
                .base_api(Method::GET, "dags")
                .await?
                .query(&[("limit", limit.to_string()), ("offset", offset.to_string())])
                .send()
                .await?
                .error_for_status()?;

            let page: model::dag::DagList = response.json().await?;

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
        self.base_api(Method::PATCH, &format!("dags/{dag_id}"))
            .await?
            .json(&serde_json::json!({"is_paused": !is_paused}))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn fetch_dag_code(&self, dag_id: &str) -> Result<String> {
        let r = self
            .base_api(Method::GET, &format!("dagSources/{dag_id}"))
            .await?
            .build()?;
        let response = self.base.client.execute(r).await?.error_for_status()?;
        let dag_source: model::dag::DagSource = response.json().await?;
        Ok(dag_source.content)
    }
}
