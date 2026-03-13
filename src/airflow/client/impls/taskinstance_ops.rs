use anyhow::Result;
use async_trait::async_trait;

use crate::airflow::client::convert_v1::{
    v1_task_instance_collection_to_list, v1_task_instance_try_to_gantt,
};
use crate::airflow::client::convert_v2::{
    v2_task_instance_list_to_list, v2_task_instance_try_to_gantt,
};
use crate::airflow::client::FlowrsClient;
use crate::airflow::model::common::{TaskInstanceList, TaskTryGantt};
use crate::airflow::traits::TaskInstanceOperations;

#[async_trait]
impl TaskInstanceOperations for FlowrsClient {
    async fn list_task_instances(
        &self,
        dag_id: &str,
        dag_run_id: &str,
    ) -> Result<TaskInstanceList> {
        match self {
            Self::V1(client) => {
                let response = client.fetch_task_instances(dag_id, dag_run_id).await?;
                Ok(v1_task_instance_collection_to_list(response))
            }
            Self::V2(client) => {
                let response = client.fetch_task_instances(dag_id, dag_run_id).await?;
                Ok(v2_task_instance_list_to_list(response))
            }
        }
    }

    async fn list_all_taskinstances(&self) -> Result<TaskInstanceList> {
        match self {
            Self::V1(client) => {
                let response = client.fetch_all_task_instances().await?;
                Ok(v1_task_instance_collection_to_list(response))
            }
            Self::V2(client) => {
                let response = client.fetch_all_task_instances().await?;
                Ok(v2_task_instance_list_to_list(response))
            }
        }
    }

    async fn list_task_instance_tries(
        &self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
    ) -> Result<Vec<TaskTryGantt>> {
        match self {
            Self::V1(client) => {
                let response = client
                    .fetch_task_instance_tries(dag_id, dag_run_id, task_id)
                    .await?;
                Ok(response
                    .task_instances
                    .into_iter()
                    .map(v1_task_instance_try_to_gantt)
                    .collect())
            }
            Self::V2(client) => {
                let response = client
                    .fetch_task_instance_tries(dag_id, dag_run_id, task_id)
                    .await?;
                Ok(response
                    .task_instances
                    .into_iter()
                    .map(v2_task_instance_try_to_gantt)
                    .collect())
            }
        }
    }

    async fn mark_task_instance(
        &self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
        status: &str,
    ) -> Result<()> {
        match self {
            Self::V1(client) => {
                client
                    .patch_task_instance(dag_id, dag_run_id, task_id, status)
                    .await
            }
            Self::V2(client) => {
                client
                    .patch_task_instance(dag_id, dag_run_id, task_id, status)
                    .await
            }
        }
    }

    async fn clear_task_instance(
        &self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
    ) -> Result<()> {
        match self {
            Self::V1(client) => {
                client
                    .post_clear_task_instance(dag_id, dag_run_id, task_id)
                    .await
            }
            Self::V2(client) => {
                client
                    .post_clear_task_instance(dag_id, dag_run_id, task_id)
                    .await
            }
        }
    }
}
