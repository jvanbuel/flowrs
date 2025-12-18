use anyhow::Result;
use async_trait::async_trait;
use log::{debug, info};
use reqwest::{Method, Response};

use super::model;
use crate::airflow::{model::common::TaskInstanceList, traits::TaskInstanceOperations};

use super::V2Client;
const PAGE_SIZE: usize = 100;

#[async_trait]
impl TaskInstanceOperations for V2Client {
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
                )?
                .query(&[("limit", limit.to_string()), ("offset", offset.to_string())])
                .send()
                .await?
                .error_for_status()?;

            let page: model::taskinstance::TaskInstanceList = response
                .json::<model::taskinstance::TaskInstanceList>()
                .await?;

            total_entries = page.total_entries;
            let fetched_count = page.task_instances.len();
            all_task_instances.extend(page.task_instances);

            debug!(
                "Fetched {fetched_count} task instances, offset: {offset}, total: {total_entries}"
            );

            let total_usize = usize::try_from(total_entries).unwrap_or(usize::MAX);
            if fetched_count < limit || all_task_instances.len() >= total_usize {
                break;
            }

            offset += fetched_count;
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
                .base_api(Method::GET, "dags/~/dagRuns/~/taskInstances")?
                .query(&[("limit", limit.to_string()), ("offset", offset.to_string())])
                .send()
                .await?
                .error_for_status()?;

            let page: model::taskinstance::TaskInstanceList = response
                .json::<model::taskinstance::TaskInstanceList>()
                .await?;

            total_entries = page.total_entries;
            let fetched_count = page.task_instances.len();
            all_task_instances.extend(page.task_instances);

            debug!(
                "Fetched {fetched_count} task instances (all), offset: {offset}, total: {total_entries}"
            );

            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            if fetched_count < limit || all_task_instances.len() >= total_entries as usize {
                break;
            }

            offset += limit;
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
            )?
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
            .base_api(Method::POST, &format!("dags/{dag_id}/clearTaskInstances"))?
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
