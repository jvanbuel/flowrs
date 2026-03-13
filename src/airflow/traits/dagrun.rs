use anyhow::Result;
use async_trait::async_trait;

use crate::airflow::model::common::DagRunList;

/// Trait for DAG Run operations
#[async_trait]
pub trait DagRunOperations: Send + Sync {
    /// List DAG runs for a specific DAG
    async fn list_dagruns(&self, dag_id: &str) -> Result<DagRunList>;

    /// List all DAG runs across all DAGs
    #[allow(unused)]
    async fn list_all_dagruns(&self) -> Result<DagRunList>;

    /// Mark a DAG run with a specific status
    async fn mark_dag_run(&self, dag_id: &str, dag_run_id: &str, status: &str) -> Result<()>;

    /// Clear a DAG run
    async fn clear_dagrun(&self, dag_id: &str, dag_run_id: &str) -> Result<()>;

    /// Trigger a new DAG run
    async fn trigger_dag_run(&self, dag_id: &str, logical_date: Option<&str>) -> Result<()>;
}
