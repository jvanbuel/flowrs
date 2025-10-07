use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DagRunList {
    #[serde(rename = "dag_runs")]
    pub dag_runs: Vec<DagRun>,
    #[serde(rename = "total_entries")]
    pub total_entries: i64,
}

#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DagRun {
    pub dag_run_id: String,
    pub dag_id: String,
    #[serde(with = "time::serde::iso8601::option")]
    pub logical_date: Option<OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub queued_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub start_date: Option<OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub end_date: Option<OffsetDateTime>,
    pub duration: Option<f64>,
    #[serde(with = "time::serde::iso8601::option")]
    pub data_interval_start: Option<OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub data_interval_end: Option<OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub run_after: Option<OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub last_scheduling_decision: Option<OffsetDateTime>,
    pub run_type: String,
    pub state: String,
    pub triggered_by: Option<String>,
    pub triggering_user_name: Option<String>,
    pub conf: Option<serde_json::Value>,
    pub note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dag_versions: Option<Vec<DagVersion>>,
    pub bundle_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DagVersion {
    pub id: String,
    pub version_number: i64,
    pub dag_id: String,
    pub bundle_name: Option<String>,
    pub bundle_version: Option<String>,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: OffsetDateTime,
}
