use anyhow::Result;
use log::{debug, info};
use reqwest::{Method, Response};

use crate::airflow::model::taskinstance::TaskInstanceList;

use super::AirFlowClient;

impl AirFlowClient {
    pub async fn list_task_instances(
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
            .await?;
        let daglist: TaskInstanceList = response.json::<TaskInstanceList>().await?;
        info!("TaskInstances: {:?}", daglist);
        Ok(daglist)
    }

    #[allow(dead_code)]
    pub async fn list_all_taskinstances(&self) -> Result<TaskInstanceList> {
        let response: Response = self
            .base_api(Method::GET, "dags/~/dagRuns/~/taskInstances")?
            .send()
            .await?;
        let daglist: TaskInstanceList = response.json::<TaskInstanceList>().await?;
        Ok(daglist)
    }

    pub async fn mark_task_instance(
        &self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
        status: &str,
    ) -> Result<()> {
        let resp: Response = self
            .base_api(
                Method::PATCH,
                &format!("dags/{dag_id}/dagRuns/{dag_run_id}/taskInstances/{task_id}",),
            )?
            .json(&serde_json::json!({"new_state": status, "dry_run": false}))
            .send()
            .await?;
        debug!("{:?}", resp);
        Ok(())
    }

    pub async fn clear_task_instance(
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
                    "task_ids": [task_id,],
                    "dag_run_id": dag_run_id,
                    "include_downstream": true,
                    "only_failed": false,
                    "reset_dag_runs": true,
                }
            ))
            .send()
            .await?;
        debug!("{:?}", resp);
        Ok(())
    }
}
