use anyhow::Result;
use async_trait::async_trait;
use log::{debug, info};
use reqwest::Method;

use super::model;
use crate::airflow::{model::common::DagList, traits::DagOperations};

use super::V2Client;

const PAGE_SIZE: usize = 50;

#[async_trait]
impl DagOperations for V2Client {
    async fn list_dags(&self) -> Result<DagList> {
        let mut all_dags = Vec::new();
        let mut offset = 0;
        let limit = PAGE_SIZE;
        let mut total_entries;

        loop {
            let response = self
                .base_api(Method::GET, "dags")?
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
        }
        .into())
    }

    async fn toggle_dag(&self, dag_id: &str, is_paused: bool) -> Result<()> {
        self.base_api(Method::PATCH, &format!("dags/{dag_id}"))?
            .json(&serde_json::json!({"is_paused": !is_paused}))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    async fn get_dag_code(&self, dag: &crate::airflow::model::common::Dag) -> Result<String> {
        let r = self
            .base_api(Method::GET, &format!("dagSources/{}", dag.dag_id))?
            .build()?;
        let response = self.base.client.execute(r).await?.error_for_status()?;
        let dag_source: model::dag::DagSource = response.json().await?;
        Ok(dag_source.content)
    }
}
