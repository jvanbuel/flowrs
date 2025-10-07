use anyhow::Result;
use async_trait::async_trait;

use crate::airflow::model::common::DagList;

/// Trait for DAG operations
#[async_trait]
pub trait DagOperations: Send + Sync {
    /// List all DAGs
    async fn list_dags(&self) -> Result<DagList>;

    /// Toggle a DAG's paused state
    async fn toggle_dag(&self, dag_id: &str, is_paused: bool) -> Result<()>;

    /// Get DAG source code using file token
    async fn get_dag_code(&self, file_token: &str) -> Result<String>;
}
