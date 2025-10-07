use anyhow::Result;
use async_trait::async_trait;
use log::{debug, info};
use reqwest::{Method, Response};

use crate::airflow::{model::common::TaskInstanceList, traits::TaskInstanceOperations};
use super::model;

use super::V1Client;

#[async_trait]
impl TaskInstanceOperations for V1Client {
    async fn list_task_instances(
        &self,
        dag_id: &str,
        dag_run_id: &str,
    ) -> Result<TaskInstanceList> {
        let response: Response = self
            .base_api(
                Method::GET,
                &format!("dags/{dag_id}/dagRuns/{dag_run_id}/taskInstances"),
            )?
            .send()
            .await?
            .error_for_status()?;
        let daglist: model::taskinstance::TaskInstanceCollectionResponse = response
            .json::<model::taskinstance::TaskInstanceCollectionResponse>()
            .await?;
        info!("TaskInstances: {daglist:?}");
        Ok(daglist.into())
    }

    async fn list_all_taskinstances(&self) -> Result<TaskInstanceList> {
        let response: Response = self
            .base_api(Method::GET, "dags/~/dagRuns/~/taskInstances")?
            .send()
            .await?
            .error_for_status()?;

        debug!("list_all_taskinstances response: {response:?}");
        let daglist: model::taskinstance::TaskInstanceCollectionResponse = response
            .json::<model::taskinstance::TaskInstanceCollectionResponse>()
            .await?;
        Ok(daglist.into())
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
