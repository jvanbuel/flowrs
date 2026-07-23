use log::{debug, info};
use reqwest::Method;

use super::model;
use super::V2Client;
use crate::client::read_json;
use crate::error::Result;

const PAGE_SIZE: usize = 100;

impl V2Client {
    pub async fn fetch_task_instances(
        &self,
        dag_id: &str,
        dag_run_id: &str,
    ) -> Result<model::taskinstance::TaskInstanceList> {
        let mut all_task_instances = Vec::new();
        let mut offset = 0;
        let limit = PAGE_SIZE;
        let mut total_entries;

        loop {
            let request = self
                .base_api(
                    Method::GET,
                    &format!("dags/{dag_id}/dagRuns/{dag_run_id}/taskInstances"),
                )
                .await?
                .query(&[("limit", limit.to_string()), ("offset", offset.to_string())]);
            let response = self.execute(request).await?;
            let page: model::taskinstance::TaskInstanceList =
                read_json(response, "task instances response").await?;

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

        Ok(model::taskinstance::TaskInstanceList {
            task_instances: all_task_instances,
            total_entries,
        })
    }

    pub async fn fetch_all_task_instances(&self) -> Result<model::taskinstance::TaskInstanceList> {
        let mut all_task_instances = Vec::new();
        let mut offset = 0;
        let limit = 100;
        let mut total_entries;

        loop {
            let request = self
                .base_api(Method::GET, "dags/~/dagRuns/~/taskInstances")
                .await?
                .query(&[("limit", limit.to_string()), ("offset", offset.to_string())]);
            let response = self.execute(request).await?;
            let page: model::taskinstance::TaskInstanceList =
                read_json(response, "all task instances response").await?;

            total_entries = page.total_entries;
            let fetched_count = page.task_instances.len();
            all_task_instances.extend(page.task_instances);

            debug!(
                "Fetched {fetched_count} task instances (all), offset: {offset}, total: {total_entries}"
            );

            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "duration/count values from the API are small and non-negative in practice"
            )]
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

        Ok(model::taskinstance::TaskInstanceList {
            task_instances: all_task_instances,
            total_entries,
        })
    }

    pub async fn fetch_task_instance_tries(
        &self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
    ) -> Result<model::taskinstance::TaskInstanceTriesResponse> {
        let request = self
            .base_api(
                Method::GET,
                &format!("dags/{dag_id}/dagRuns/{dag_run_id}/taskInstances/{task_id}/tries"),
            )
            .await?;
        let response = self.execute(request).await?;
        let tries: model::taskinstance::TaskInstanceTriesResponse =
            read_json(response, "task instance tries response").await?;
        debug!(
            "Fetched {} tries for task {task_id}",
            tries.task_instances.len()
        );

        Ok(tries)
    }

    pub async fn patch_task_instance(
        &self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
        status: &str,
    ) -> Result<()> {
        let request = self
            .base_api(
                Method::PATCH,
                &format!("dags/{dag_id}/dagRuns/{dag_run_id}/taskInstances/{task_id}"),
            )
            .await?
            // Airflow 3's `/api/v2` PatchTaskInstanceBody uses a strict schema that
            // forbids unknown fields; unlike the `/api/v1` endpoint it does not accept
            // `dry_run` (dry-run is a separate endpoint), so sending it yields a 422.
            .json(&serde_json::json!({"new_state": status}));
        let resp = self.execute(request).await?;
        debug!("{resp:?}");
        Ok(())
    }

    pub async fn post_clear_task_instance(
        &self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
    ) -> Result<()> {
        let request = self
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
            ));
        let resp = self.execute(request).await?;
        debug!("{resp:?}");
        Ok(())
    }
}
