use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskInstanceList {
    #[serde(rename = "task_instances")]
    pub task_instances: Vec<TaskInstance>,
    #[serde(rename = "total_entries")]
    pub total_entries: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskInstance {
    pub id: String,
    #[serde(rename = "task_id")]
    pub task_id: String,
    #[serde(rename = "dag_id")]
    pub dag_id: String,
    #[serde(rename = "dag_run_id")]
    pub dag_run_id: String,
    #[serde(rename = "map_index")]
    pub map_index: i64,
    #[serde(rename = "logical_date", with = "time::serde::iso8601::option")]
    pub logical_date: Option<OffsetDateTime>,
    #[serde(rename = "run_after", with = "time::serde::iso8601")]
    pub run_after: OffsetDateTime,
    #[serde(rename = "start_date", with = "time::serde::iso8601::option")]
    pub start_date: Option<OffsetDateTime>,
    #[serde(rename = "end_date", with = "time::serde::iso8601::option")]
    pub end_date: Option<OffsetDateTime>,
    pub duration: Option<f64>,
    pub state: Option<String>,
    #[serde(rename = "try_number")]
    pub try_number: i64,
    #[serde(rename = "max_tries")]
    pub max_tries: i64,
    #[serde(rename = "task_display_name")]
    pub task_display_name: String,
    #[serde(rename = "dag_display_name")]
    pub dag_display_name: String,
    pub hostname: Option<String>,
    pub unixname: Option<String>,
    pub pool: String,
    #[serde(rename = "pool_slots")]
    pub pool_slots: i64,
    pub queue: Option<String>,
    #[serde(rename = "priority_weight")]
    pub priority_weight: Option<i64>,
    pub operator: Option<String>,
    #[serde(rename = "operator_name")]
    pub operator_name: Option<String>,
    #[serde(rename = "queued_when", with = "time::serde::iso8601::option")]
    pub queued_when: Option<OffsetDateTime>,
    #[serde(rename = "scheduled_when", with = "time::serde::iso8601::option")]
    pub scheduled_when: Option<OffsetDateTime>,
    pub pid: Option<i64>,
    pub executor: Option<String>,
    #[serde(rename = "executor_config")]
    pub executor_config: String,
    pub note: Option<String>,
    #[serde(rename = "rendered_map_index")]
    pub rendered_map_index: Option<String>,
    #[serde(rename = "rendered_fields")]
    pub rendered_fields: serde_json::Value,
    pub trigger: Option<Trigger>,
    #[serde(rename = "triggerer_job")]
    pub triggerer_job: Option<Job>,
    #[serde(rename = "dag_version")]
    pub dag_version: Option<DagVersion>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trigger {
    pub id: i64,
    pub classpath: String,
    pub kwargs: String,
    #[serde(rename = "created_date", with = "time::serde::iso8601")]
    pub created_date: OffsetDateTime,
    #[serde(rename = "triggerer_id")]
    pub triggerer_id: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: i64,
    #[serde(rename = "dag_id")]
    pub dag_id: Option<String>,
    pub state: Option<String>,
    #[serde(rename = "job_type")]
    pub job_type: Option<String>,
    #[serde(rename = "start_date", with = "time::serde::iso8601::option")]
    pub start_date: Option<OffsetDateTime>,
    #[serde(rename = "end_date", with = "time::serde::iso8601::option")]
    pub end_date: Option<OffsetDateTime>,
    #[serde(rename = "latest_heartbeat", with = "time::serde::iso8601::option")]
    pub latest_heartbeat: Option<OffsetDateTime>,
    #[serde(rename = "executor_class")]
    pub executor_class: Option<String>,
    pub hostname: Option<String>,
    pub unixname: Option<String>,
    #[serde(rename = "dag_display_name")]
    pub dag_display_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DagVersion {
    pub id: String,
    #[serde(rename = "version_number")]
    pub version_number: i64,
    #[serde(rename = "dag_id")]
    pub dag_id: String,
    #[serde(rename = "bundle_name")]
    pub bundle_name: Option<String>,
    #[serde(rename = "bundle_version")]
    pub bundle_version: Option<String>,
    #[serde(rename = "created_at", with = "time::serde::iso8601")]
    pub created_at: OffsetDateTime,
    #[serde(rename = "dag_display_name")]
    pub dag_display_name: String,
}
