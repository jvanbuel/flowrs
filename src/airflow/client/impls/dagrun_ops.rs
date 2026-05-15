use anyhow::Result;
use async_trait::async_trait;

use crate::airflow::client::convert_v1::v1_dagrun_collection_to_list;
use crate::airflow::client::convert_v2::v2_dagrun_list_to_list;
use crate::airflow::client::FlowrsClient;
use crate::airflow::model::common::DagRunList;
use crate::airflow::traits::DagRunOperations;

#[async_trait]
impl DagRunOperations for FlowrsClient {
    async fn list_dagruns(&self, dag_id: &str) -> Result<DagRunList> {
        match self {
            Self::V1(client) => {
                let response = client.fetch_dagruns(dag_id).await?;
                Ok(v1_dagrun_collection_to_list(response))
            }
            Self::V2(client) => {
                let response = client.fetch_dagruns(dag_id).await?;
                Ok(v2_dagrun_list_to_list(response))
            }
        }
    }

    async fn list_all_dagruns(&self) -> Result<DagRunList> {
        match self {
            Self::V1(client) => {
                let response = client.fetch_all_dagruns().await?;
                Ok(v1_dagrun_collection_to_list(response))
            }
            Self::V2(client) => {
                let response = client.fetch_all_dagruns().await?;
                Ok(v2_dagrun_list_to_list(response))
            }
        }
    }

    async fn mark_dag_run(&self, dag_id: &str, dag_run_id: &str, status: &str) -> Result<()> {
        match self {
            Self::V1(client) => client.patch_dag_run(dag_id, dag_run_id, status).await,
            Self::V2(client) => client.patch_dag_run(dag_id, dag_run_id, status).await,
        }
    }

    async fn clear_dagrun(&self, dag_id: &str, dag_run_id: &str) -> Result<()> {
        match self {
            Self::V1(client) => client.post_clear_dagrun(dag_id, dag_run_id).await,
            Self::V2(client) => client.post_clear_dagrun(dag_id, dag_run_id).await,
        }
    }

    async fn trigger_dag_run(
        &self,
        dag_id: &str,
        logical_date: Option<&str>,
        conf: Option<serde_json::Value>,
    ) -> Result<()> {
        match self {
            Self::V1(client) => {
                client
                    .post_trigger_dag_run(dag_id, logical_date, conf)
                    .await
            }
            Self::V2(client) => {
                client
                    .post_trigger_dag_run(dag_id, logical_date, conf)
                    .await
            }
        }
    }
}
