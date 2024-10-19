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
    #[serde(rename = "default_view")]
    pub default_view: String,
    pub description: Option<String>,
    #[serde(rename = "file_token")]
    pub file_token: String,
    pub fileloc: String,
    #[serde(rename = "has_import_errors")]
    pub has_import_errors: bool,
    #[serde(rename = "has_task_concurrency_limits")]
    pub has_task_concurrency_limits: bool,
    #[serde(rename = "is_active")]
    pub is_active: bool,
    #[serde(rename = "is_paused")]
    pub is_paused: bool,
    #[serde(rename = "is_subdag")]
    pub is_subdag: bool,
    #[serde(rename = "last_expired", with = "time::serde::iso8601::option")]
    pub last_expired: Option<OffsetDateTime>,
    #[serde(rename = "last_parsed_time", with = "time::serde::iso8601::option")]
    pub last_parsed_time: Option<OffsetDateTime>,
    #[serde(rename = "last_pickled", with = "time::serde::iso8601::option")]
    pub last_pickled: Option<OffsetDateTime>,
    #[serde(rename = "max_active_runs")]
    pub max_active_runs: i64,
    #[serde(rename = "max_active_tasks")]
    pub max_active_tasks: i64,
    #[serde(rename = "next_dagrun", with = "time::serde::iso8601::option")]
    pub next_dagrun: Option<OffsetDateTime>,
    #[serde(
        rename = "next_dagrun_create_after",
        with = "time::serde::iso8601::option"
    )]
    pub next_dagrun_create_after: Option<OffsetDateTime>,
    #[serde(
        rename = "next_dagrun_data_interval_end",
        with = "time::serde::iso8601::option"
    )]
    pub next_dagrun_data_interval_end: Option<OffsetDateTime>,
    #[serde(
        rename = "next_dagrun_data_interval_start",
        with = "time::serde::iso8601::option"
    )]
    pub next_dagrun_data_interval_start: Option<OffsetDateTime>,
    pub owners: Vec<String>,
    #[serde(rename = "pickle_id")]
    pub pickle_id: Option<String>,
    #[serde(rename = "root_dag_id")]
    pub root_dag_id: Option<String>,
    #[serde(rename = "schedule_interval")]
    pub schedule_interval: Option<ScheduleInterval>,
    #[serde(rename = "scheduler_lock")]
    pub scheduler_lock: Option<bool>,
    pub tags: Vec<Tag>,
    #[serde(rename = "timetable_description")]
    pub timetable_description: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScheduleInterval {
    pub value: Option<String>,
    __type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    #[serde(rename = "name")]
    pub name: String,
}
