use anyhow::Result;
use async_trait::async_trait;
use log::{debug, info};
use reqwest::{Method, Response};

use super::model;
use flowrs_airflow_model::model::common::taskinstance::TaskInstanceState;
use flowrs_airflow_model::model::common::{TaskInstance, TaskInstanceList, TaskTryGantt};
use flowrs_airflow_model::traits::TaskInstanceOperations;

use super::V1Client;

const PAGE_SIZE: usize = 100;

#[async_trait]
impl TaskInstanceOperations for V1Client {
    async fn list_task_instances(
        &self,
        dag_id: &str,
        dag_run_id: &str,
    ) -> Result<TaskInstanceList> {
        let mut all_task_instances = Vec::new();
        let mut offset = 0;
        let limit = PAGE_SIZE;
        let mut total_entries;

        loop {
            let response: Response = self
                .base_api(
                    Method::GET,
                    &format!("dags/{dag_id}/dagRuns/{dag_run_id}/taskInstances"),
                )
                .await?
                .query(&[("limit", limit.to_string()), ("offset", offset.to_string())])
                .send()
                .await?
                .error_for_status()?;

            let page: model::taskinstance::TaskInstanceCollectionResponse = response
                .json::<model::taskinstance::TaskInstanceCollectionResponse>()
                .await?;

            total_entries = page.total_entries;
            let fetched_count = page.task_instances.len();
            all_task_instances.extend(page.task_instances);

            debug!(
                "Fetched {fetched_count} task instances, offset: {offset}, total: {total_entries}"
            );

            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            if fetched_count < limit || all_task_instances.len() >= total_entries as usize {
                break;
            }

            offset += limit;
        }

        info!(
            "Fetched total {} task instances out of {}",
            all_task_instances.len(),
            total_entries
        );

        Ok(TaskInstanceList {
            task_instances: all_task_instances.into_iter().map(Into::into).collect(),
            total_entries,
        })
    }

    async fn list_all_taskinstances(&self) -> Result<TaskInstanceList> {
        let mut all_task_instances = Vec::new();
        let mut offset = 0;
        let limit = 100;
        let mut total_entries;

        loop {
            let response: Response = self
                .base_api(Method::GET, "dags/~/dagRuns/~/taskInstances")
                .await?
                .query(&[("limit", limit.to_string()), ("offset", offset.to_string())])
                .send()
                .await?
                .error_for_status()?;

            let page: model::taskinstance::TaskInstanceCollectionResponse = response
                .json::<model::taskinstance::TaskInstanceCollectionResponse>()
                .await?;

            total_entries = page.total_entries;
            let fetched_count = page.task_instances.len();
            all_task_instances.extend(page.task_instances);

            debug!("Fetched {fetched_count} task instances (all), offset: {offset}, total: {total_entries}");

            let total_usize = usize::try_from(total_entries).unwrap_or(usize::MAX);
            if fetched_count < limit || all_task_instances.len() >= total_usize {
                break;
            }

            offset += fetched_count;
        }

        info!(
            "Fetched total {} task instances (all) out of {}",
            all_task_instances.len(),
            total_entries
        );

        Ok(TaskInstanceList {
            task_instances: all_task_instances.into_iter().map(Into::into).collect(),
            total_entries,
        })
    }

    async fn list_task_instance_tries(
        &self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
    ) -> Result<Vec<TaskTryGantt>> {
        let response: Response = self
            .base_api(
                Method::GET,
                &format!("dags/{dag_id}/dagRuns/{dag_run_id}/taskInstances/{task_id}/tries"),
            )
            .await?
            .send()
            .await?
            .error_for_status()?;

        let tries: model::taskinstance::TaskInstanceTriesResponse = response.json().await?;
        debug!(
            "Fetched {} tries for task {task_id}",
            tries.task_instances.len()
        );

        Ok(tries.task_instances.into_iter().map(Into::into).collect())
    }

    async fn mark_task_instance(
        &self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
        status: &str,
    ) -> Result<()> {
        let resp: Response = self
            .base_api(
                Method::PATCH,
                &format!("dags/{dag_id}/dagRuns/{dag_run_id}/taskInstances/{task_id}"),
            )
            .await?
            .json(&serde_json::json!({"new_state": status, "dry_run": false}))
            .send()
            .await?
            .error_for_status()?;
        debug!("{resp:?}");
        Ok(())
    }

    async fn clear_task_instance(
        &self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
    ) -> Result<()> {
        let resp: Response = self
            .base_api(Method::POST, &format!("dags/{dag_id}/clearTaskInstances"))
            .await?
            .json(&serde_json::json!(
                {
                    "dry_run": false,
                    "task_ids": [task_id],
                    "dag_run_id": dag_run_id,
                    "include_downstream": true,
                    "only_failed": false,
                    "reset_dag_runs": true,
                }
            ))
            .send()
            .await?
            .error_for_status()?;
        debug!("{resp:?}");
        Ok(())
    }
}

// From trait implementations for v1 models
impl From<model::taskinstance::TaskInstanceResponse> for TaskInstance {
    fn from(value: model::taskinstance::TaskInstanceResponse) -> Self {
        Self {
            task_id: value.task_id.into(),
            dag_id: value.dag_id.into(),
            dag_run_id: value.dag_run_id.into(),
            logical_date: Some(value.execution_date),
            start_date: value.start_date,
            end_date: value.end_date,
            duration: value.duration,
            state: value.state.map(|s| TaskInstanceState::from(s.as_str())),
            try_number: value.try_number,
            max_tries: value.max_tries,
            map_index: value.map_index,
            hostname: Some(value.hostname),
            unixname: Some(value.unixname),
            pool: value.pool,
            pool_slots: value.pool_slots,
            queue: value.queue,
            priority_weight: value.priority_weight,
            operator: value.operator,
            queued_when: value.queued_when,
            pid: value.pid,
            note: value.note,
        }
    }
}

impl From<model::taskinstance::TaskInstanceCollectionResponse> for TaskInstanceList {
    fn from(value: model::taskinstance::TaskInstanceCollectionResponse) -> Self {
        Self {
            task_instances: value
                .task_instances
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
            total_entries: value.total_entries,
        }
    }
}

impl From<model::taskinstance::TaskInstanceTryResponse> for TaskTryGantt {
    fn from(value: model::taskinstance::TaskInstanceTryResponse) -> Self {
        Self {
            try_number: value.try_number,
            start_date: value.start_date,
            end_date: value.end_date,
            state: value.state.map(|s| TaskInstanceState::from(s.as_str())),
        }
    }
}
