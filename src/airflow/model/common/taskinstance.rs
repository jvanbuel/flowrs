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

impl TaskInstanceState {
    /// The wire name of this state, as the Airflow API spells it.
    ///
    /// A static lookup so filters and renderers can compare or display the
    /// state without formatting into a fresh `String`.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Running => "running",
            Self::Failed => "failed",
            Self::Queued => "queued",
            Self::UpForRetry => "up_for_retry",
            Self::UpForReschedule => "up_for_reschedule",
            Self::Skipped => "skipped",
            Self::Deferred => "deferred",
            Self::Removed => "removed",
            Self::Restarting => "restarting",
            Self::UpstreamFailed => "upstream_failed",
            Self::Scheduled => "scheduled",
            Self::Unknown => "unknown",
        }
    }
}

impl fmt::Display for TaskInstanceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
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
    /// When the scheduler moved the task to the `scheduled` state.
    /// Only available from the Airflow v3 (`/api/v2`) API; `None` for v2 servers.
    pub scheduled_when: Option<OffsetDateTime>,
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
