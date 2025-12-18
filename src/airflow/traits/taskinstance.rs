use anyhow::Result;
use async_trait::async_trait;

use crate::airflow::model::common::TaskInstanceList;

/// Trait for Task Instance operations
#[async_trait]
pub trait TaskInstanceOperations: Send + Sync {
    /// List task instances for a specific DAG run
    async fn list_task_instances(&self, dag_id: &str, dag_run_id: &str)
        -> Result<TaskInstanceList>;

    /// List all task instances across all DAG runs
    #[allow(unused)]
    async fn list_all_taskinstances(&self) -> Result<TaskInstanceList>;

    /// Mark a task instance with a specific status
    async fn mark_task_instance(
        &self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
        status: &str,
    ) -> Result<()>;

    /// Clear a task instance
    async fn clear_task_instance(
        &self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
    ) -> Result<()>;
}
