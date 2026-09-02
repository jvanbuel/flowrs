use std::fmt;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::duration::TimeBounded;
use super::{DagId, DagRunId};

/// State of a DAG run as reported by the Airflow API.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DagRunState {
    Success,
    Running,
    Failed,
    Queued,
    UpForRetry,
    /// Catch-all for unknown/future states returned by the API.
    #[default]
    #[serde(other)]
    Unknown,
}

impl DagRunState {
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
            Self::Unknown => "unknown",
        }
    }
}

impl fmt::Display for DagRunState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for DagRunState {
    fn from(s: &str) -> Self {
        match s {
            "success" => Self::Success,
            "running" => Self::Running,
            "failed" => Self::Failed,
            "queued" => Self::Queued,
            "up_for_retry" => Self::UpForRetry,
            _ => Self::Unknown,
        }
    }
}

/// The type of a DAG run (how it was triggered).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunType {
    Scheduled,
    Manual,
    Backfill,
    DatasetTriggered,
    AssetTriggered,
    /// Catch-all for unknown/future run types returned by the API.
    #[default]
    #[serde(other)]
    Unknown,
}

impl RunType {
    /// The wire name of this state, as the Airflow API spells it.
    ///
    /// A static lookup so filters and renderers can compare or display the
    /// state without formatting into a fresh `String`.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Scheduled => "scheduled",
            Self::Manual => "manual",
            Self::Backfill => "backfill",
            Self::DatasetTriggered => "dataset_triggered",
            Self::AssetTriggered => "asset_triggered",
            Self::Unknown => "unknown",
        }
    }
}

impl fmt::Display for RunType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for RunType {
    fn from(s: &str) -> Self {
        match s {
            "scheduled" => Self::Scheduled,
            "manual" => Self::Manual,
            "backfill" => Self::Backfill,
            "dataset_triggered" => Self::DatasetTriggered,
            "asset_triggered" => Self::AssetTriggered,
            _ => Self::Unknown,
        }
    }
}

/// Common `DagRun` model used by the application
#[allow(
    clippy::struct_field_names,
    reason = "field names mirror the Airflow API response schema"
)]
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DagRun {
    pub dag_id: DagId,
    pub dag_run_id: DagRunId,
    pub logical_date: Option<OffsetDateTime>,
    pub data_interval_end: Option<OffsetDateTime>,
    pub data_interval_start: Option<OffsetDateTime>,
    pub end_date: Option<OffsetDateTime>,
    pub start_date: Option<OffsetDateTime>,
    pub last_scheduling_decision: Option<OffsetDateTime>,
    pub run_type: RunType,
    pub state: DagRunState,
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

    fn is_running(&self) -> bool {
        self.state == DagRunState::Running || self.state == DagRunState::Queued
    }
}
