use anyhow::Result;
use async_trait::async_trait;

use crate::airflow::client::convert_v1::v1_task_collection_to_list;
use crate::airflow::client::convert_v2::v2_task_collection_to_list;
use crate::airflow::client::FlowrsClient;
use crate::airflow::model::common::TaskList;
use crate::airflow::traits::TaskOperations;

#[async_trait]
impl TaskOperations for FlowrsClient {
    async fn list_tasks(&self, dag_id: &str) -> Result<TaskList> {
        match self {
            Self::V1(client) => {
                let response = client.fetch_tasks(dag_id).await?;
                Ok(v1_task_collection_to_list(response))
            }
            Self::V2(client) => {
                let response = client.fetch_tasks(dag_id).await?;
                Ok(v2_task_collection_to_list(response))
            }
        }
    }
}
