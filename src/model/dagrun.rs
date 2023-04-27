use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
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
    #[serde(rename = "data_interval_end")]
    pub data_interval_end: String,
    #[serde(rename = "data_interval_start")]
    pub data_interval_start: String,
    #[serde(rename = "end_date")]
    pub end_date: String,
    #[serde(rename = "execution_date")]
    pub execution_date: String,
    #[serde(rename = "external_trigger")]
    pub external_trigger: bool,
    #[serde(rename = "last_scheduling_decision")]
    pub last_scheduling_decision: String,
    #[serde(rename = "logical_date")]
    pub logical_date: String,
    pub note: Option<String>,
    #[serde(rename = "run_type")]
    pub run_type: String,
    #[serde(rename = "start_date")]
    pub start_date: String,
    pub state: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conf {}
