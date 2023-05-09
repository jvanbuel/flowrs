use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskInstanceList {
    #[serde(rename = "task_instances")]
    pub task_instances: Vec<TaskInstance>,
    #[serde(rename = "total_entries")]
    pub total_entries: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskInstance {
    #[serde(rename = "task_id")]
    pub task_id: String,
    #[serde(rename = "dag_id")]
    pub dag_id: String,
    #[serde(rename = "dag_run_id")]
    pub dag_run_id: String,
    #[serde(rename = "execution_date")]
    pub execution_date: DateTime<Utc>,
    #[serde(rename = "start_date")]
    pub start_date: DateTime<Utc>,
    #[serde(rename = "end_date")]
    pub end_date: DateTime<Utc>,
    pub duration: Option<f64>,
    pub state: String,
    #[serde(rename = "try_number")]
    pub try_number: i64,
    #[serde(rename = "map_index")]
    pub map_index: i64,
    #[serde(rename = "max_tries")]
    pub max_tries: i64,
    pub hostname: String,
    pub unixname: String,
    pub pool: String,
    #[serde(rename = "pool_slots")]
    pub pool_slots: i64,
    pub queue: String,
    #[serde(rename = "priority_weight")]
    pub priority_weight: i64,
    pub operator: String,
    #[serde(rename = "queued_when")]
    pub queued_when: Option<DateTime<Utc>>,
    pub pid: Option<i64>,
    #[serde(rename = "executor_config")]
    pub executor_config: String,
    #[serde(rename = "sla_miss")]
    pub sla_miss: Option<SlaMiss>,
    #[serde(rename = "rendered_fields")]
    pub rendered_fields: RenderedFields,
    pub trigger: Option<Trigger>,
    #[serde(rename = "triggerer_job")]
    pub triggerer_job: Option<TriggererJob>,
    pub note: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlaMiss {
    #[serde(rename = "task_id")]
    pub task_id: String,
    #[serde(rename = "dag_id")]
    pub dag_id: String,
    #[serde(rename = "execution_date")]
    pub execution_date: DateTime<Utc>,
    #[serde(rename = "email_sent")]
    pub email_sent: bool,
    pub timestamp: DateTime<Utc>,
    pub description: String,
    #[serde(rename = "notification_sent")]
    pub notification_sent: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderedFields {}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trigger {
    pub id: i64,
    pub classpath: String,
    pub kwargs: String,
    #[serde(rename = "created_date")]
    pub created_date: DateTime<Utc>,
    #[serde(rename = "triggerer_id")]
    pub triggerer_id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggererJob {
    pub id: i64,
    #[serde(rename = "dag_id")]
    pub dag_id: String,
    pub state: String,
    #[serde(rename = "job_type")]
    pub job_type: String,
    #[serde(rename = "start_date")]
    pub start_date: DateTime<Utc>,
    #[serde(rename = "end_date")]
    pub end_date: DateTime<Utc>,
    #[serde(rename = "latest_heartbeat")]
    pub latest_heartbeat: DateTime<Utc>,
    #[serde(rename = "executor_class")]
    pub executor_class: String,
    pub hostname: String,
    pub unixname: String,
}
