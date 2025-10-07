use anyhow::Result;
use async_trait::async_trait;

use crate::airflow::model::common::DagStatsResponse;

/// Trait for DAG Statistics operations
#[async_trait]
pub trait DagStatsOperations: Send + Sync {
    /// Get DAG statistics for the specified DAG IDs
    async fn get_dag_stats(&self, dag_ids: Vec<&str>) -> Result<DagStatsResponse>;
}
