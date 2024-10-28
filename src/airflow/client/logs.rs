use reqwest::Method;

use crate::airflow::model::log::Log;
use crate::airflow::model::taskinstance::TaskInstance;
use crate::app::error::Result;

use super::AirFlowClient;

impl AirFlowClient {
    pub async fn get_task_logs(&self, task_instance: &TaskInstance) -> Result<Log> {
        let reponse = self
            .base_api(
                Method::GET,
                format!(
                    "dags/{}/dagRuns/{}/taskInstances/{}/logs",
                    task_instance.dag_id, task_instance.dag_run_id, task_instance.task_id
                )
                .as_str(),
            )?
            .send()
            .await?;
        let log = reponse.json::<Log>().await?;
        Ok(log)
    }
}
