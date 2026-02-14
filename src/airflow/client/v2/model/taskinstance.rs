use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::dagrun::DagVersion;
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskInstanceList {
    #[serde(rename = "task_instances")]
    pub task_instances: Vec<TaskInstance>,
    #[serde(rename = "total_entries")]
    pub total_entries: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskInstance {
    pub id: String,
    pub task_id: String,
    pub dag_id: String,
    pub dag_run_id: String,
    pub map_index: i64,
    #[serde(with = "time::serde::iso8601::option")]
    pub logical_date: Option<OffsetDateTime>,
    #[serde(with = "time::serde::iso8601")]
    pub run_after: OffsetDateTime,
    #[serde(with = "time::serde::iso8601::option")]
    pub start_date: Option<OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub end_date: Option<OffsetDateTime>,
    pub duration: Option<f64>,
    pub state: Option<String>,
    pub try_number: u32,
    pub max_tries: i64,
    pub task_display_name: String,
    pub hostname: Option<String>,
    pub unixname: Option<String>,
    pub pool: String,
    pub pool_slots: i64,
    pub queue: Option<String>,
    pub priority_weight: Option<i64>,
    pub operator: Option<String>,
    pub operator_name: Option<String>,
    #[serde(with = "time::serde::iso8601::option")]
    pub queued_when: Option<OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub scheduled_when: Option<OffsetDateTime>,
    pub pid: Option<i64>,
    pub executor: Option<String>,
    pub executor_config: String,
    pub note: Option<String>,
    pub rendered_map_index: Option<String>,
    pub rendered_fields: serde_json::Value,
    pub trigger: Option<Trigger>,
    pub triggerer_job: Option<Job>,
    pub dag_version: Option<DagVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trigger {
    pub id: i64,
    pub classpath: String,
    pub kwargs: String,
    #[serde(with = "time::serde::iso8601")]
    pub created_date: OffsetDateTime,
    pub triggerer_id: Option<i64>,
}

#[allow(clippy::struct_field_names)]
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Job {
    pub id: i64,
    pub dag_id: Option<String>,
    pub state: Option<String>,
    pub job_type: Option<String>,
    #[serde(with = "time::serde::iso8601::option")]
    pub start_date: Option<OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub end_date: Option<OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub latest_heartbeat: Option<OffsetDateTime>,
    pub executor_class: Option<String>,
    pub hostname: Option<String>,
    pub unixname: Option<String>,
}
