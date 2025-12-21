use crate::airflow::client::v1;
use crate::airflow::client::v2;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Common DAG model used by the application
#[allow(clippy::struct_field_names)]
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dag {
    pub dag_id: String,
    pub dag_display_name: Option<String>,
    pub description: Option<String>,
    pub fileloc: String,
    pub is_paused: bool,
    pub is_active: Option<bool>,
    pub has_import_errors: bool,
    pub has_task_concurrency_limits: bool,
    pub last_parsed_time: Option<OffsetDateTime>,
    pub last_expired: Option<OffsetDateTime>,
    pub max_active_tasks: i64,
    pub max_active_runs: Option<i64>,
    pub next_dagrun_logical_date: Option<OffsetDateTime>,
    pub next_dagrun_data_interval_start: Option<OffsetDateTime>,
    pub next_dagrun_data_interval_end: Option<OffsetDateTime>,
    pub next_dagrun_create_after: Option<OffsetDateTime>,
    pub owners: Vec<String>,
    pub tags: Vec<Tag>,
    pub file_token: String,
    pub timetable_description: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DagList {
    pub dags: Vec<Dag>,
    pub total_entries: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
}

// From trait implementations for v1 models
impl From<v1::model::dag::DagResponse> for Dag {
    fn from(value: v1::model::dag::DagResponse) -> Self {
        Self {
            dag_id: value.dag_id,
            dag_display_name: Some(value.dag_display_name),
            description: value.description,
            fileloc: value.fileloc,
            is_paused: value.is_paused.unwrap_or(false),
            is_active: value.is_active,
            has_import_errors: value.has_import_errors.unwrap_or(false),
            has_task_concurrency_limits: value.has_task_concurrency_limits.unwrap_or(false),
            last_parsed_time: value.last_parsed_time,
            last_expired: value.last_expired,
            max_active_tasks: value.max_active_tasks.unwrap_or(0),
            max_active_runs: value.max_active_runs,
            next_dagrun_logical_date: value.next_dagrun,
            next_dagrun_data_interval_start: value.next_dagrun_data_interval_start,
            next_dagrun_data_interval_end: value.next_dagrun_data_interval_end,
            next_dagrun_create_after: value.next_dagrun_create_after,
            owners: value.owners.clone(),
            tags: value
                .tags
                .unwrap_or_default()
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
            file_token: value.file_token.clone(),
            timetable_description: value.timetable_description,
        }
    }
}

impl From<v1::model::dag::DagCollectionResponse> for DagList {
    fn from(value: v1::model::dag::DagCollectionResponse) -> Self {
        Self {
            dags: value
                .dags
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
            total_entries: value.total_entries,
        }
    }
}

impl From<v1::model::dag::DagTagResponse> for Tag {
    fn from(value: v1::model::dag::DagTagResponse) -> Self {
        Self { name: value.name }
    }
}

// From trait implementations for v2 models
impl From<v2::model::dag::Dag> for Dag {
    fn from(value: v2::model::dag::Dag) -> Self {
        Self {
            dag_id: value.dag_id,
            dag_display_name: Some(value.dag_display_name),
            description: value.description,
            fileloc: value.fileloc,
            is_paused: value.is_paused,
            is_active: None,
            has_import_errors: value.has_import_errors,
            has_task_concurrency_limits: value.has_task_concurrency_limits,
            last_parsed_time: value.last_parsed_time,
            last_expired: value.last_expired,
            max_active_tasks: value.max_active_tasks,
            max_active_runs: value.max_active_runs,
            next_dagrun_logical_date: value.next_dagrun_logical_date,
            next_dagrun_create_after: value.next_dagrun_run_after,
            next_dagrun_data_interval_start: value.next_dagrun_data_interval_start,
            next_dagrun_data_interval_end: value.next_dagrun_data_interval_end,
            owners: value.owners,
            tags: value
                .tags
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
            file_token: value.file_token,
            timetable_description: value.timetable_description,
        }
    }
}

impl From<v2::model::dag::DagList> for DagList {
    fn from(value: v2::model::dag::DagList) -> Self {
        Self {
            dags: value
                .dags
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
            total_entries: value.total_entries,
        }
    }
}

impl From<v2::model::dag::Tag> for Tag {
    fn from(value: v2::model::dag::Tag) -> Self {
        Self { name: value.name }
    }
}
