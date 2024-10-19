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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DagRun {
    pub conf: Conf,
    #[serde(rename = "dag_id")]
    pub dag_id: String,
    #[serde(rename = "dag_run_id")]
    pub dag_run_id: String,
    #[serde(rename = "data_interval_end", with = "time::serde::iso8601::option")]
    pub data_interval_end: Option<OffsetDateTime>,
    #[serde(rename = "data_interval_start", with = "time::serde::iso8601::option")]
    pub data_interval_start: Option<OffsetDateTime>,
    #[serde(rename = "end_date", with = "time::serde::iso8601::option")]
    pub end_date: Option<OffsetDateTime>,
    #[serde(rename = "execution_date", with = "time::serde::iso8601::option")]
    pub execution_date: Option<OffsetDateTime>,
    #[serde(rename = "external_trigger")]
    pub external_trigger: bool,
    #[serde(
        rename = "last_scheduling_decision",
        with = "time::serde::iso8601::option"
    )]
    pub last_scheduling_decision: Option<OffsetDateTime>,
    #[serde(rename = "logical_date", with = "time::serde::iso8601::option")]
    pub logical_date: Option<OffsetDateTime>,
    pub note: Option<String>,
    #[serde(rename = "run_type")]
    pub run_type: String,
    #[serde(rename = "start_date", with = "time::serde::iso8601::option")]
    pub start_date: Option<OffsetDateTime>,
    pub state: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conf {}
