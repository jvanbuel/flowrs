use anyhow::Result;
use async_trait::async_trait;

use crate::airflow::client::convert_v1::v1_dagstats_to_response;
use crate::airflow::client::convert_v2::v2_dagstats_to_response;
use crate::airflow::client::FlowrsClient;
use crate::airflow::model::common::DagStatsResponse;
use crate::airflow::traits::DagStatsOperations;

#[async_trait]
impl DagStatsOperations for FlowrsClient {
    async fn get_dag_stats(&self, dag_ids: Vec<&str>) -> Result<DagStatsResponse> {
        match self {
            Self::V1(client) => {
                let response = client.fetch_dag_stats(dag_ids).await?;
                Ok(v1_dagstats_to_response(response))
            }
            Self::V2(client) => {
                let response = client.fetch_dag_stats(dag_ids).await?;
                Ok(v2_dagstats_to_response(response))
            }
        }
    }
}
