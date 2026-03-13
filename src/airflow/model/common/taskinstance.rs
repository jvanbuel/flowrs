use std::fmt;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::duration::TimeBounded;
use super::{DagId, DagRunId, TaskId};

/// State of a task instance as reported by the Airflow API.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskInstanceState {
    Success,
    Running,
    Failed,
    Queued,
    UpForRetry,
    UpForReschedule,
    Skipped,
    Deferred,
    Removed,
    Restarting,
    UpstreamFailed,
    Scheduled,
    /// Catch-all for unknown/future states returned by the API.
    #[default]
    #[serde(other)]
    Unknown,
}

impl fmt::Display for TaskInstanceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Running => write!(f, "running"),
            Self::Failed => write!(f, "failed"),
            Self::Queued => write!(f, "queued"),
            Self::UpForRetry => write!(f, "up_for_retry"),
            Self::UpForReschedule => write!(f, "up_for_reschedule"),
            Self::Skipped => write!(f, "skipped"),
            Self::Deferred => write!(f, "deferred"),
            Self::Removed => write!(f, "removed"),
            Self::Restarting => write!(f, "restarting"),
            Self::UpstreamFailed => write!(f, "upstream_failed"),
            Self::Scheduled => write!(f, "scheduled"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<&str> for TaskInstanceState {
    fn from(s: &str) -> Self {
        match s {
            "success" => Self::Success,
            "running" => Self::Running,
            "failed" => Self::Failed,
            "queued" => Self::Queued,
            "up_for_retry" => Self::UpForRetry,
            "up_for_reschedule" => Self::UpForReschedule,
            "skipped" => Self::Skipped,
            "deferred" => Self::Deferred,
            "removed" => Self::Removed,
            "restarting" => Self::Restarting,
            "upstream_failed" => Self::UpstreamFailed,
            "scheduled" => Self::Scheduled,
            _ => Self::Unknown,
        }
    }
}

/// Common `TaskInstance` model used by the application
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskInstance {
    pub task_id: TaskId,
    pub dag_id: DagId,
    pub dag_run_id: DagRunId,
    pub logical_date: Option<OffsetDateTime>,
    pub start_date: Option<OffsetDateTime>,
    pub end_date: Option<OffsetDateTime>,
    pub duration: Option<f64>,
    pub state: Option<TaskInstanceState>,
    pub try_number: u32,
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

impl TimeBounded for TaskInstance {
    fn start_date(&self) -> Option<OffsetDateTime> {
        self.start_date
    }

    fn end_date(&self) -> Option<OffsetDateTime> {
        self.end_date
    }

    fn is_running(&self) -> bool {
        matches!(
            self.state,
            Some(
                TaskInstanceState::Running
                    | TaskInstanceState::Queued
                    | TaskInstanceState::Scheduled
                    | TaskInstanceState::Deferred
                    | TaskInstanceState::Restarting
                    | TaskInstanceState::UpForReschedule
                    | TaskInstanceState::UpForRetry
            )
        )
    }
}

