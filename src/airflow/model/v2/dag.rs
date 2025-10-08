use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DagList {
    pub dags: Vec<Dag>,
    pub total_entries: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dag {
    #[serde(rename = "dag_id")]
    pub dag_id: String,
    #[serde(rename = "dag_display_name")]
    pub dag_display_name: String,
    #[serde(rename = "is_paused")]
    pub is_paused: bool,
    #[serde(rename = "is_stale")]
    pub is_stale: bool,
    #[serde(rename = "last_parsed_time", with = "time::serde::iso8601::option")]
    pub last_parsed_time: Option<OffsetDateTime>,
    #[serde(rename = "last_parse_duration")]
    pub last_parse_duration: Option<f64>,
    #[serde(rename = "last_expired", with = "time::serde::iso8601::option")]
    pub last_expired: Option<OffsetDateTime>,
    #[serde(rename = "bundle_name")]
    pub bundle_name: Option<String>,
    #[serde(rename = "bundle_version")]
    pub bundle_version: Option<String>,
    #[serde(rename = "relative_fileloc")]
    pub relative_fileloc: Option<String>,
    pub fileloc: String,
    pub description: Option<String>,
    #[serde(rename = "timetable_summary")]
    pub timetable_summary: Option<String>,
    #[serde(rename = "timetable_description")]
    pub timetable_description: Option<String>,
    pub tags: Vec<Tag>,
    #[serde(rename = "max_active_tasks")]
    pub max_active_tasks: i64,
    #[serde(rename = "max_active_runs")]
    pub max_active_runs: Option<i64>,
    #[serde(rename = "max_consecutive_failed_dag_runs")]
    pub max_consecutive_failed_dag_runs: i64,
    #[serde(rename = "has_task_concurrency_limits")]
    pub has_task_concurrency_limits: bool,
    #[serde(rename = "has_import_errors")]
    pub has_import_errors: bool,
    #[serde(
        rename = "next_dagrun_logical_date",
        with = "time::serde::iso8601::option"
    )]
    pub next_dagrun_logical_date: Option<OffsetDateTime>,
    #[serde(
        rename = "next_dagrun_data_interval_start",
        with = "time::serde::iso8601::option"
    )]
    pub next_dagrun_data_interval_start: Option<OffsetDateTime>,
    #[serde(
        rename = "next_dagrun_data_interval_end",
        with = "time::serde::iso8601::option"
    )]
    pub next_dagrun_data_interval_end: Option<OffsetDateTime>,
    #[serde(
        rename = "next_dagrun_run_after",
        with = "time::serde::iso8601::option"
    )]
    pub next_dagrun_run_after: Option<OffsetDateTime>,
    #[serde(rename = "owners")]
    pub owners: Vec<String>,
    #[serde(rename = "file_token")]
    pub file_token: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
}
