use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskInstanceCollectionResponse {
    pub task_instances: Vec<TaskInstanceResponse>,
    pub total_entries: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskInstanceResponse {
    pub task_id: String,
    pub task_display_name: String,
    pub dag_id: String,
    pub dag_run_id: String,
    #[serde(with = "time::serde::iso8601")]
    pub execution_date: OffsetDateTime,
    #[serde(with = "time::serde::iso8601::option")]
    pub start_date: Option<OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub end_date: Option<OffsetDateTime>,
    pub duration: Option<f64>,
    pub state: Option<String>,
    pub try_number: i64,
    pub map_index: i64,
    pub max_tries: i64,
    pub hostname: String,
    pub unixname: String,
    pub pool: String,
    pub pool_slots: i64,
    pub queue: Option<String>,
    pub priority_weight: Option<i64>,
    pub operator: Option<String>,
    #[serde(with = "time::serde::iso8601::option")]
    pub queued_when: Option<OffsetDateTime>,
    pub pid: Option<i64>,
    pub executor: Option<String>,
    pub executor_config: String,
    pub sla_miss: Option<serde_json::Value>,
    pub rendered_map_index: Option<String>,
    pub rendered_fields: serde_json::Value,
    pub trigger: Option<TriggerResponse>,
    pub triggerer_job: Option<JobResponse>,
    pub note: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TriggerResponse {
    pub id: i64,
    pub classpath: String,
    pub kwargs: String,
    #[serde(with = "time::serde::iso8601::option")]
    pub created_date: Option<OffsetDateTime>,
    pub triggerer_id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JobResponse {
    pub id: i64,
    pub dag_id: String,
    pub state: String,
    pub job_type: String,
    #[serde(with = "time::serde::iso8601::option")]
    pub start_date: Option<OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub end_date: Option<OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub latest_heartbeat: Option<OffsetDateTime>,
    pub executor_class: String,
    pub hostname: String,
    pub unixname: String,
}
