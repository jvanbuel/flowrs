use anyhow::Result;
use async_trait::async_trait;
use log::{debug, info};
use reqwest::Method;

use super::model;
use flowrs_airflow_model::model::common::{Dag, DagList, Tag};
use flowrs_airflow_model::traits::DagOperations;

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
        }
        .into())
    }

    async fn toggle_dag(&self, dag_id: &str, is_paused: bool) -> Result<()> {
        self.base_api(Method::PATCH, &format!("dags/{dag_id}"))
            .await?
            .json(&serde_json::json!({"is_paused": !is_paused}))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    async fn get_dag_code(&self, dag: &Dag) -> Result<String> {
        let r = self
            .base_api(Method::GET, &format!("dagSources/{}", dag.dag_id))
            .await?
            .build()?;
        let response = self.base.client.execute(r).await?.error_for_status()?;
        let dag_source: model::dag::DagSource = response.json().await?;
        Ok(dag_source.content)
    }
}

// From trait implementations for v2 models
impl From<model::dag::Dag> for Dag {
    fn from(value: model::dag::Dag) -> Self {
        Self {
            dag_id: value.dag_id.into(),
            dag_display_name: Some(value.dag_display_name),
            description: value.description,
            fileloc: value.fileloc,
            is_paused: value.is_paused,
            is_active: None,
            has_import_errors: value.has_import_errors,
            has_task_concurrency_limits: value.has_task_concurrency_limits,
            last_parsed_time: value.last_parsed_time,
            last_expired: value.last_expired,
            max_active_tasks: value.max_active_tasks,
            max_active_runs: value.max_active_runs,
            next_dagrun_logical_date: value.next_dagrun_logical_date,
            next_dagrun_create_after: value.next_dagrun_run_after,
            next_dagrun_data_interval_start: value.next_dagrun_data_interval_start,
            next_dagrun_data_interval_end: value.next_dagrun_data_interval_end,
            owners: value.owners,
            tags: value
                .tags
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
            file_token: value.file_token,
            timetable_description: value.timetable_description,
        }
    }
}

impl From<model::dag::DagList> for DagList {
    fn from(value: model::dag::DagList) -> Self {
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

impl From<model::dag::Tag> for Tag {
    fn from(value: model::dag::Tag) -> Self {
        Self { name: value.name }
    }
}
