use anyhow::Result;
use async_trait::async_trait;

use crate::airflow::model::common::TaskList;

/// Trait for task definition operations
#[async_trait]
pub trait TaskOperations: Send + Sync {
    /// List all tasks for a DAG
    async fn list_tasks(&self, dag_id: &str) -> Result<TaskList>;
}
