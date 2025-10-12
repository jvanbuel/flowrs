use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Common DagRun model used by the application
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DagRunList {
    pub dag_runs: Vec<DagRun>,
    pub total_entries: i64,
}

// From trait implementations for v1 models
impl From<crate::airflow::model::v1::dagrun::DAGRunResponse> for DagRun {
    /// Converts a v1 `DAGRunResponse` into the common `DagRun` representation.
    ///
    /// The resulting `DagRun` copies the corresponding fields from the v1 response and sets
    /// `external_trigger` to `Some(value.external_trigger)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::airflow::model::v1::dagrun::DAGRunResponse;
    /// use crate::airflow::model::common::dagrun::DagRun;
    ///
    /// let v1 = DAGRunResponse {
    ///     dag_id: "example".into(),
    ///     dag_run_id: Some("r1".into()),
    ///     logical_date: None,
    ///     data_interval_end: None,
    ///     data_interval_start: None,
    ///     end_date: None,
    ///     start_date: None,
    ///     last_scheduling_decision: None,
    ///     run_type: "manual".into(),
    ///     state: "running".into(),
    ///     note: None,
    ///     external_trigger: false,
    /// };
    ///
    /// let common: DagRun = v1.into();
    /// assert_eq!(common.dag_id, "example");
    /// assert_eq!(common.dag_run_id, "r1");
    /// assert_eq!(common.external_trigger, Some(false));
    /// ```
    fn from(value: crate::airflow::model::v1::dagrun::DAGRunResponse) -> Self {
        DagRun {
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

impl From<crate::airflow::model::v1::dagrun::DAGRunCollectionResponse> for DagRunList {
    /// Converts a v1 `DAGRunCollectionResponse` into a `DagRunList`.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::airflow::model::v1::dagrun::DAGRunCollectionResponse;
    /// use crate::airflow::model::common::dagrun::DagRunList;
    ///
    /// let v1 = DAGRunCollectionResponse { dag_runs: vec![], total_entries: 0 };
    /// let list: DagRunList = v1.into();
    /// assert_eq!(list.total_entries, 0);
    /// ```
    fn from(value: crate::airflow::model::v1::dagrun::DAGRunCollectionResponse) -> Self {
        DagRunList {
            dag_runs: value.dag_runs.into_iter().map(|dr| dr.into()).collect(),
            total_entries: value.total_entries,
        }
    }
}

// From trait implementations for v2 models
impl From<crate::airflow::model::v2::dagrun::DagRun> for DagRun {
    /// Converts a v2 `DagRun` into the common `DagRun` representation.
    ///
    /// The v2 fields are mapped directly; the `external_trigger` field is set to `None` on the resulting common `DagRun`.
    ///
    /// # Examples
    ///
    /// ```
    /// // Construct a v2 DagRun (fields shown illustratively; actual construction may vary)
    /// let v2 = crate::airflow::model::v2::dagrun::DagRun {
    ///     dag_id: "example".to_string(),
    ///     dag_run_id: "run_1".to_string(),
    ///     ..Default::default()
    /// };
    /// let common: crate::airflow::model::common::dagrun::DagRun = v2.into();
    /// assert_eq!(common.dag_id, "example");
    /// assert_eq!(common.dag_run_id, "run_1");
    /// assert_eq!(common.external_trigger, None);
    /// ```
    fn from(value: crate::airflow::model::v2::dagrun::DagRun) -> Self {
        DagRun {
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

impl From<crate::airflow::model::v2::dagrun::DagRunList> for DagRunList {
    /// Converts a v2 `DagRunList` into the common `DagRunList` representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::airflow::model::v2::dagrun::DagRunList as V2DagRunList;
    /// use crate::airflow::model::common::dagrun::DagRunList;
    ///
    /// let v2 = V2DagRunList { dag_runs: vec![], total_entries: 0 };
    /// let common: DagRunList = v2.into();
    /// assert_eq!(common.total_entries, 0);
    /// ```
    fn from(value: crate::airflow::model::v2::dagrun::DagRunList) -> Self {
        DagRunList {
            dag_runs: value.dag_runs.into_iter().map(|dr| dr.into()).collect(),
            total_entries: value.total_entries,
        }
    }
}