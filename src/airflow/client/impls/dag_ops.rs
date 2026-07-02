use anyhow::Result;
use async_trait::async_trait;

use crate::airflow::client::convert_v1::v1_dag_collection_to_dag_list;
use crate::airflow::client::convert_v2::v2_dag_list_to_dag_list;
use crate::airflow::client::FlowrsClient;
use crate::airflow::model::common::{Dag, DagList};
use crate::airflow::traits::DagOperations;

#[async_trait]
impl DagOperations for FlowrsClient {
    async fn list_dags(&self) -> Result<DagList> {
        match self {
            Self::V1(client) => {
                let response = client.fetch_dags().await?;
                Ok(v1_dag_collection_to_dag_list(response))
            }
            Self::V2(client) => {
                let response = client.fetch_dags().await?;
                Ok(v2_dag_list_to_dag_list(response))
            }
        }
    }

    async fn toggle_dag(&self, dag_id: &str, is_paused: bool) -> Result<()> {
        match self {
            Self::V1(client) => client.patch_dag_pause(dag_id, is_paused).await,
            Self::V2(client) => client.patch_dag_pause(dag_id, is_paused).await,
        }
    }

    async fn get_dag_code(&self, dag: &Dag) -> Result<String> {
        match self {
            Self::V1(client) => client.fetch_dag_code(&dag.file_token).await,
            Self::V2(client) => client.fetch_dag_code(&dag.dag_id).await,
        }
    }

    async fn get_dag_params(&self, dag_id: &str) -> Result<Option<serde_json::Value>> {
        match self {
            Self::V1(client) => client.fetch_dag_params(dag_id).await,
            Self::V2(client) => client.fetch_dag_params(dag_id).await,
        }
    }
}
