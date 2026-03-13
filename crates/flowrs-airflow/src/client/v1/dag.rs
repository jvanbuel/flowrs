use anyhow::Result;
use async_trait::async_trait;
use log::{debug, info};
use reqwest::Method;

use flowrs_airflow_model::model::common::{Dag, DagList, Tag};
use flowrs_airflow_model::traits::DagOperations;

use super::model::dag::DagCollectionResponse;

use super::V1Client;

const PAGE_SIZE: usize = 50;

#[async_trait]
impl DagOperations for V1Client {
    async fn list_dags(&self) -> Result<DagList> {
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

            let page: DagCollectionResponse = response.json().await?;

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
        }
        .into())
    }

    async fn toggle_dag(&self, dag_id: &str, is_paused: bool) -> Result<()> {
        self.base_api(Method::PATCH, &format!("dags/{dag_id}"))
            .await?
            .query(&[("update_mask", "is_paused")])
            .json(&serde_json::json!({"is_paused": !is_paused}))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    async fn get_dag_code(&self, dag: &Dag) -> Result<String> {
        let r = self
            .base_api(Method::GET, &format!("dagSources/{}", dag.file_token))
            .await?
            .build()?;
        let response = self.base.client.execute(r).await?.error_for_status()?;
        let code = response.text().await?;
        Ok(code)
    }
}

// From trait implementations for v1 models
impl From<super::model::dag::DagResponse> for Dag {
    fn from(value: super::model::dag::DagResponse) -> Self {
        Self {
            dag_id: value.dag_id.into(),
            dag_display_name: Some(value.dag_display_name),
            description: value.description,
            fileloc: value.fileloc,
            is_paused: value.is_paused.unwrap_or(false),
            is_active: value.is_active,
            has_import_errors: value.has_import_errors.unwrap_or(false),
            has_task_concurrency_limits: value.has_task_concurrency_limits.unwrap_or(false),
            last_parsed_time: value.last_parsed_time,
            last_expired: value.last_expired,
            max_active_tasks: value.max_active_tasks.unwrap_or(0),
            max_active_runs: value.max_active_runs,
            next_dagrun_logical_date: value.next_dagrun,
            next_dagrun_data_interval_start: value.next_dagrun_data_interval_start,
            next_dagrun_data_interval_end: value.next_dagrun_data_interval_end,
            next_dagrun_create_after: value.next_dagrun_create_after,
            owners: value.owners.clone(),
            tags: value
                .tags
                .unwrap_or_default()
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
            file_token: value.file_token.clone(),
            timetable_description: value.timetable_description,
        }
    }
}

impl From<DagCollectionResponse> for DagList {
    fn from(value: DagCollectionResponse) -> Self {
        Self {
            dags: value
                .dags
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
            total_entries: value.total_entries,
        }
    }
}

impl From<super::model::dag::DagTagResponse> for Tag {
    fn from(value: super::model::dag::DagTagResponse) -> Self {
        Self { name: value.name }
    }
}
