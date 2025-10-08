use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DagCollectionResponse {
    pub dags: Vec<DagResponse>,
    pub total_entries: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DagResponse {
    pub dag_id: String,
    pub dag_display_name: String,
    pub root_dag_id: Option<String>,
    pub is_paused: Option<bool>,
    pub is_active: Option<bool>,
    pub is_subdag: bool,
    #[serde(with = "time::serde::iso8601::option")]
    pub last_parsed_time: Option<OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub last_pickled: Option<OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub last_expired: Option<OffsetDateTime>,
    pub scheduler_lock: Option<bool>,
    pub pickle_id: Option<String>,
    pub default_view: Option<String>,
    pub fileloc: String,
    pub file_token: String,
    pub owners: Vec<String>,
    pub description: Option<String>,
    pub schedule_interval: Option<serde_json::Value>,
    pub timetable_description: Option<String>,
    pub tags: Option<Vec<DagTagResponse>>,
    pub max_active_tasks: Option<i64>,
    pub max_active_runs: Option<i64>,
    pub has_task_concurrency_limits: Option<bool>,
    pub has_import_errors: Option<bool>,
    #[serde(with = "time::serde::iso8601::option")]
    pub next_dagrun: Option<OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub next_dagrun_data_interval_start: Option<OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub next_dagrun_data_interval_end: Option<OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub next_dagrun_create_after: Option<OffsetDateTime>,
    pub max_consecutive_failed_dag_runs: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DagTagResponse {
    pub name: String,
}
