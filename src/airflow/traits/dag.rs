use anyhow::Result;
use async_trait::async_trait;

use crate::airflow::model::common::{Dag, DagList};

/// Trait for DAG operations
#[async_trait]
pub trait DagOperations: Send + Sync {
    /// List all DAGs
    async fn list_dags(&self) -> Result<DagList>;

    /// Toggle a DAG's paused state
    async fn toggle_dag(&self, dag_id: &str, is_paused: bool) -> Result<()>;

    /// Get DAG source code (uses `file_token` in v1, `dag_id` in v2)
    async fn get_dag_code(&self, dag: &Dag) -> Result<String>;
}
