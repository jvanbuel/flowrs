use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Common TaskInstance model used by the application
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskInstance {
    pub task_id: String,
    pub dag_id: String,
    pub dag_run_id: String,
    pub logical_date: Option<OffsetDateTime>,
    pub start_date: Option<OffsetDateTime>,
    pub end_date: Option<OffsetDateTime>,
    pub duration: Option<f64>,
    pub state: Option<String>,
    pub try_number: i64,
    pub max_tries: i64,
    pub map_index: i64,
    pub hostname: Option<String>,
    pub unixname: Option<String>,
    pub pool: String,
    pub pool_slots: i64,
    pub queue: Option<String>,
    pub priority_weight: Option<i64>,
    pub operator: Option<String>,
    pub queued_when: Option<OffsetDateTime>,
    pub pid: Option<i64>,
    pub note: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskInstanceList {
    pub task_instances: Vec<TaskInstance>,
    pub total_entries: i64,
}

// From trait implementations for v1 models
impl From<crate::airflow::model::v1::taskinstance::TaskInstanceResponse> for TaskInstance {
    /// Convert a v1 `TaskInstanceResponse` into the shared `TaskInstance` representation.
    ///
    /// The conversion maps each field from the v1 response into the corresponding field of the
    /// common `TaskInstance`, wrapping values in `Option` where the common model expects them.
    ///
    /// # Examples
    ///
    /// ```
    /// // Given a v1 response `resp`, convert it into the shared model:
    /// // let resp: crate::airflow::model::v1::taskinstance::TaskInstanceResponse = ...;
    /// // let ti: crate::airflow::shared::TaskInstance = resp.into();
    /// ```
    fn from(value: crate::airflow::model::v1::taskinstance::TaskInstanceResponse) -> Self {
        TaskInstance {
            task_id: value.task_id,
            dag_id: value.dag_id,
            dag_run_id: value.dag_run_id,
            logical_date: Some(value.execution_date),
            start_date: value.start_date,
            end_date: value.end_date,
            duration: value.duration,
            state: value.state,
            try_number: value.try_number,
            max_tries: value.max_tries,
            map_index: value.map_index,
            hostname: Some(value.hostname),
            unixname: Some(value.unixname),
            pool: value.pool,
            pool_slots: value.pool_slots,
            queue: value.queue,
            priority_weight: value.priority_weight,
            operator: value.operator,
            queued_when: value.queued_when,
            pid: value.pid,
            note: value.note,
        }
    }
}

impl From<crate::airflow::model::v1::taskinstance::TaskInstanceCollectionResponse> for TaskInstanceList {
    /// Convert a v1 `TaskInstanceCollectionResponse` into the shared `TaskInstanceList`.
    ///
    /// # Examples
    ///
    /// ```
    /// let v1 = crate::airflow::model::v1::taskinstance::TaskInstanceCollectionResponse {
    ///     task_instances: Vec::new(),
    ///     total_entries: 0,
    /// };
    /// let list: crate::mymodule::TaskInstanceList = v1.into();
    /// assert_eq!(list.total_entries, 0);
    /// ```
    fn from(value: crate::airflow::model::v1::taskinstance::TaskInstanceCollectionResponse) -> Self {
        TaskInstanceList {
            task_instances: value.task_instances.into_iter().map(|ti| ti.into()).collect(),
            total_entries: value.total_entries,
        }
    }
}

// From trait implementations for v2 models
impl From<crate::airflow::model::v2::taskinstance::TaskInstance> for TaskInstance {
    /// Convert a v2 Airflow `TaskInstance` into the shared `TaskInstance` model.
    ///
    /// Returns a `TaskInstance` populated with the corresponding fields from the provided v2 model.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::airflow::model::v2::taskinstance as v2;
    /// // construct a v2 TaskInstance (fields omitted for brevity)
    /// let v2_ti = v2::TaskInstance {
    ///     task_id: "task".to_string(),
    ///     dag_id: "dag".to_string(),
    ///     dag_run_id: "run".to_string(),
    ///     logical_date: None,
    ///     start_date: None,
    ///     end_date: None,
    ///     duration: None,
    ///     state: None,
    ///     try_number: 0,
    ///     max_tries: 0,
    ///     map_index: -1,
    ///     hostname: None,
    ///     unixname: None,
    ///     pool: String::new(),
    ///     pool_slots: 0,
    ///     queue: None,
    ///     priority_weight: None,
    ///     operator: None,
    ///     queued_when: None,
    ///     pid: None,
    ///     note: None,
    /// };
    ///
    /// let common: crate::airflow::shared::taskinstance::TaskInstance = v2_ti.into();
    /// assert_eq!(common.task_id, "task");
    /// ```
    fn from(value: crate::airflow::model::v2::taskinstance::TaskInstance) -> Self {
        TaskInstance {
            task_id: value.task_id,
            dag_id: value.dag_id,
            dag_run_id: value.dag_run_id,
            logical_date: value.logical_date,
            start_date: value.start_date,
            end_date: value.end_date,
            duration: value.duration,
            state: value.state,
            try_number: value.try_number,
            max_tries: value.max_tries,
            map_index: value.map_index,
            hostname: value.hostname,
            unixname: value.unixname,
            pool: value.pool,
            pool_slots: value.pool_slots,
            queue: value.queue,
            priority_weight: value.priority_weight,
            operator: value.operator,
            queued_when: value.queued_when,
            pid: value.pid,
            note: value.note,
        }
    }
}

impl From<crate::airflow::model::v2::taskinstance::TaskInstanceList> for TaskInstanceList {
    /// Convert an Airflow v2 `TaskInstanceList` into the shared `TaskInstanceList` model.
    ///
    /// # Examples
    ///
    /// ```
    /// let v2 = crate::airflow::model::v2::taskinstance::TaskInstanceList {
    ///     task_instances: vec![],
    ///     total_entries: 0,
    /// };
    /// let list: super::TaskInstanceList = v2.into();
    /// assert_eq!(list.total_entries, 0);
    /// ```
    fn from(value: crate::airflow::model::v2::taskinstance::TaskInstanceList) -> Self {
        TaskInstanceList {
            task_instances: value.task_instances.into_iter().map(|ti| ti.into()).collect(),
            total_entries: value.total_entries,
        }
    }
}