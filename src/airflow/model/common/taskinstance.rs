use crate::airflow::client::v1;
use crate::airflow::client::v2;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Common `TaskInstance` model used by the application
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
impl From<v1::model::taskinstance::TaskInstanceResponse> for TaskInstance {
    fn from(value: v1::model::taskinstance::TaskInstanceResponse) -> Self {
        Self {
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

impl From<v1::model::taskinstance::TaskInstanceCollectionResponse> for TaskInstanceList {
    fn from(value: v1::model::taskinstance::TaskInstanceCollectionResponse) -> Self {
        Self {
            task_instances: value
                .task_instances
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
            total_entries: value.total_entries,
        }
    }
}

// From trait implementations for v2 models
impl From<v2::model::taskinstance::TaskInstance> for TaskInstance {
    fn from(value: v2::model::taskinstance::TaskInstance) -> Self {
        Self {
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

impl From<v2::model::taskinstance::TaskInstanceList> for TaskInstanceList {
    fn from(value: v2::model::taskinstance::TaskInstanceList) -> Self {
        Self {
            task_instances: value
                .task_instances
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
            total_entries: value.total_entries,
        }
    }
}
