use std::error::Error;

use reqwest::{Method, Response};

use crate::model::taskinstance::TaskInstanceList;

use super::client::AirFlowClient;

impl AirFlowClient {
    pub async fn list_task_instances(
        &self,
        dag_id: &str,
        dag_run_id: &str,
    ) -> Result<TaskInstanceList, Box<dyn Error + Send + Sync>> {
        let response: Response = self
            .base_api(
                Method::GET,
                &format!("dags/{dag_id}/dagRuns/{dag_run_id}/taskInstances"),
            )?
            .send()
            .await?;
        let daglist: TaskInstanceList = response.json::<TaskInstanceList>().await?;
        Ok(daglist)
    }

    pub async fn list_all_taskinstances(
        &self,
    ) -> Result<TaskInstanceList, Box<dyn Error + Send + Sync>> {
        let response: Response = self
            .base_api(Method::GET, "dags/~/dagRuns/~/taskInstances")?
            .send()
            .await?;
        let daglist: TaskInstanceList = response.json::<TaskInstanceList>().await?;
        Ok(daglist)
    }
}
