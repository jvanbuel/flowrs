use anyhow::Result;

use crate::airflow::client::convert_v1::v1_log_to_log;
use crate::airflow::client::convert_v2::v2_log_to_log;
use crate::airflow::client::FlowrsClient;
use crate::airflow::model::common::Log;

impl FlowrsClient {
    pub async fn get_task_logs(
        &self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
        task_try: u32,
    ) -> Result<Log> {
        match self {
            Self::V1(client) => {
                let response = client
                    .fetch_task_logs(dag_id, dag_run_id, task_id, task_try)
                    .await?;
                Ok(v1_log_to_log(response))
            }
            Self::V2(client) => {
                let response = client
                    .fetch_task_logs(dag_id, dag_run_id, task_id, task_try)
                    .await?;
                Ok(v2_log_to_log(response))
            }
        }
    }
}
