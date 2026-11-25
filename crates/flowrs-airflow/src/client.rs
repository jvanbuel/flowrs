pub mod auth;
pub mod base;
pub mod v1;
pub mod v2;

use reqwest::Response;
use serde::de::DeserializeOwned;

use crate::error::{AirflowError, Result};

pub use base::BaseClient;
pub use v1::V1Client;
pub use v2::V2Client;

/// Deserialize a response body, keeping a snippet of it in the error on failure.
///
/// The body is read as text first rather than using `Response::json`, because some
/// deployments return payloads that do not match the documented schema (older v2.x
/// versions omit `dag_display_name`, for instance) and the raw body is what makes
/// those failures diagnosable.
/// Build the `taskInstances/...` path for one task instance, relative to the API root.
///
/// Airflow uses `map_index = -1` for unmapped tasks. Mapped instances (from
/// `.expand()`) get their index appended as an extra path segment, which both
/// `/api/v1` and `/api/v2` accept.
pub(crate) fn task_instance_path(
    dag_id: &str,
    dag_run_id: &str,
    task_id: &str,
    map_index: i64,
) -> String {
    let path = format!("dags/{dag_id}/dagRuns/{dag_run_id}/taskInstances/{task_id}");
    if map_index >= 0 {
        format!("{path}/{map_index}")
    } else {
        path
    }
}

pub(crate) async fn read_json<T: DeserializeOwned>(response: Response, context: &str) -> Result<T> {
    let body = response.text().await?;
    serde_json::from_str(&body).map_err(|e| AirflowError::decode(context, &body, e))
}

#[cfg(test)]
mod tests {
    use super::task_instance_path;

    #[test]
    fn task_instance_path_appends_map_index_only_for_mapped_tasks() {
        assert_eq!(
            task_instance_path("d", "r", "t", -1),
            "dags/d/dagRuns/r/taskInstances/t"
        );
        assert_eq!(
            task_instance_path("d", "r", "t", 3),
            "dags/d/dagRuns/r/taskInstances/t/3"
        );
    }
}
