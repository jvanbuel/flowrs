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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DagRun {
    #[serde(rename = "dag_run_id")]
    pub dag_run_id: String,
    #[serde(rename = "dag_id")]
    pub dag_id: String,
    #[serde(rename = "logical_date", with = "time::serde::iso8601::option")]
    pub logical_date: Option<OffsetDateTime>,
    #[serde(rename = "queued_at", with = "time::serde::iso8601::option")]
    pub queued_at: Option<OffsetDateTime>,
    #[serde(rename = "start_date", with = "time::serde::iso8601::option")]
    pub start_date: Option<OffsetDateTime>,
    #[serde(rename = "end_date", with = "time::serde::iso8601::option")]
    pub end_date: Option<OffsetDateTime>,
    pub duration: Option<f64>,
    #[serde(rename = "data_interval_start", with = "time::serde::iso8601::option")]
    pub data_interval_start: Option<OffsetDateTime>,
    #[serde(rename = "data_interval_end", with = "time::serde::iso8601::option")]
    pub data_interval_end: Option<OffsetDateTime>,
    #[serde(rename = "run_after", with = "time::serde::iso8601")]
    pub run_after: OffsetDateTime,
    #[serde(
        rename = "last_scheduling_decision",
        with = "time::serde::iso8601::option"
    )]
    pub last_scheduling_decision: Option<OffsetDateTime>,
    #[serde(rename = "run_type")]
    pub run_type: String,
    pub state: String,
    #[serde(rename = "triggered_by")]
    pub triggered_by: Option<String>,
    #[serde(rename = "triggering_user_name")]
    pub triggering_user_name: Option<String>,
    pub conf: Option<serde_json::Value>,
    pub note: Option<String>,
    #[serde(rename = "dag_versions")]
    pub dag_versions: Vec<DagVersion>,
    #[serde(rename = "bundle_version")]
    pub bundle_version: Option<String>,
    #[serde(rename = "dag_display_name")]
    pub dag_display_name: String,
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
