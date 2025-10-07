use anyhow::Result;
use async_trait::async_trait;
use log::{debug, info};
use reqwest::{Method, Response};

use crate::airflow::{model::common::TaskInstanceList, traits::TaskInstanceOperations};
use super::model;

use super::V2Client;

#[async_trait]
impl TaskInstanceOperations for V2Client {
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
        let daglist: model::taskinstance::TaskInstanceList = response
            .json::<model::taskinstance::TaskInstanceList>()
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
        let daglist: model::taskinstance::TaskInstanceList = response
            .json::<model::taskinstance::TaskInstanceList>()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airflow::{
        client::base::BaseClient,
        config::AirflowVersion,
        managed_services::conveyor::get_conveyor_environment_servers,
        traits::{DagOperations, DagRunOperations},
    };

    fn get_test_client() -> V2Client {
        let servers = get_conveyor_environment_servers().unwrap();
        let server = servers
            .into_iter()
            .find(|s| s.version == AirflowVersion::V3)
            .unwrap();
        let base = BaseClient::new(server).unwrap();
        V2Client::new(base)
    }

    #[tokio::test]
    // TODO: use a docker-compose Airflow v3 setup for testing instead
    async fn test_list_task_instances() {
        // Skip test if CONVEYOR_TESTS is not set to avoid panicking when credentials are unavailable
        if std::env::var("CONVEYOR_TESTS").unwrap_or_default() != "true" {
            eprintln!("Skipping test_list_task_instances: Set CONVEYOR_TESTS=true to run Conveyor integration tests");
            return;
        }

        let client = get_test_client();
        let dags = client.list_dags().await.unwrap();
        let dag_id = &dags.dags[0].dag_id;
        let dagruns = client.list_dagruns(dag_id).await.unwrap();
        let dag_run_id = &dagruns.dag_runs[0].dag_run_id;
        let task_instances = client.list_task_instances(dag_id, dag_run_id).await.unwrap();
        assert!(!task_instances.task_instances.is_empty());
    }
}
