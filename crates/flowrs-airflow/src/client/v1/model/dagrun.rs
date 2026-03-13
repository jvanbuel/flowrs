use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DAGRunCollectionResponse {
    #[serde(rename = "dag_runs")]
    pub dag_runs: Vec<DAGRunResponse>,
    #[serde(rename = "total_entries")]
    pub total_entries: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DAGRunResponse {
    pub dag_run_id: Option<String>,
    pub dag_id: String,
    #[serde(with = "time::serde::iso8601::option")]
    pub logical_date: Option<OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub execution_date: Option<OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub start_date: Option<OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub end_date: Option<OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub data_interval_start: Option<OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub data_interval_end: Option<OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub last_scheduling_decision: Option<OffsetDateTime>,
    pub run_type: String,
    pub state: String,
    pub external_trigger: bool,
    pub conf: Option<serde_json::Value>,
    pub note: Option<String>,
}
