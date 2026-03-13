use anyhow::Result;
use async_trait::async_trait;

use crate::airflow::model::common::Log;

/// Trait for Log operations
#[async_trait]
pub trait LogOperations: Send + Sync {
    /// Get task logs for a specific task instance and try number
    async fn get_task_logs(
        &self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
        task_try: u32,
    ) -> Result<Log>;
}
