use crate::airflow::client::v1;
use crate::airflow::client::v2;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::duration::TimeBounded;

/// Common `DagRun` model used by the application
#[allow(clippy::struct_field_names)]
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DagRun {
    pub dag_id: String,
    pub dag_run_id: String,
    pub logical_date: Option<OffsetDateTime>,
    pub data_interval_end: Option<OffsetDateTime>,
    pub data_interval_start: Option<OffsetDateTime>,
    pub end_date: Option<OffsetDateTime>,
    pub start_date: Option<OffsetDateTime>,
    pub last_scheduling_decision: Option<OffsetDateTime>,
    pub run_type: String,
    pub state: String,
    pub note: Option<String>,
    pub external_trigger: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DagRunList {
    pub dag_runs: Vec<DagRun>,
    pub total_entries: i64,
}

impl TimeBounded for DagRun {
    fn start_date(&self) -> Option<OffsetDateTime> {
        self.start_date
    }

    fn end_date(&self) -> Option<OffsetDateTime> {
        self.end_date
    }
}

// From trait implementations for v1 models
impl From<v1::model::dagrun::DAGRunResponse> for DagRun {
    fn from(value: v1::model::dagrun::DAGRunResponse) -> Self {
        Self {
            dag_id: value.dag_id,
            dag_run_id: value.dag_run_id.unwrap_or_default(),
            logical_date: value.logical_date,
            data_interval_end: value.data_interval_end,
            data_interval_start: value.data_interval_start,
            end_date: value.end_date,
            start_date: value.start_date,
            last_scheduling_decision: value.last_scheduling_decision,
            run_type: value.run_type,
            state: value.state,
            note: value.note,
            external_trigger: Some(value.external_trigger),
        }
    }
}

impl From<v1::model::dagrun::DAGRunCollectionResponse> for DagRunList {
    fn from(value: v1::model::dagrun::DAGRunCollectionResponse) -> Self {
        Self {
            dag_runs: value
                .dag_runs
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
            total_entries: value.total_entries,
        }
    }
}

// From trait implementations for v2 models
impl From<v2::model::dagrun::DagRun> for DagRun {
    fn from(value: v2::model::dagrun::DagRun) -> Self {
        Self {
            dag_id: value.dag_id,
            dag_run_id: value.dag_run_id,
            logical_date: value.logical_date,
            data_interval_end: value.data_interval_end,
            data_interval_start: value.data_interval_start,
            end_date: value.end_date,
            start_date: value.start_date,
            last_scheduling_decision: value.last_scheduling_decision,
            run_type: value.run_type,
            state: value.state,
            note: value.note,
            external_trigger: None,
        }
    }
}

impl From<v2::model::dagrun::DagRunList> for DagRunList {
    fn from(value: v2::model::dagrun::DagRunList) -> Self {
        Self {
            dag_runs: value
                .dag_runs
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
            total_entries: value.total_entries,
        }
    }
}
