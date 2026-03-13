use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::DagId;

/// Common DAG model used by the application
#[allow(clippy::struct_field_names)]
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dag {
    pub dag_id: DagId,
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
