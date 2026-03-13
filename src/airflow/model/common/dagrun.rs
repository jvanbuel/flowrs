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

impl fmt::Display for DagRunState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Running => write!(f, "running"),
            Self::Failed => write!(f, "failed"),
            Self::Queued => write!(f, "queued"),
            Self::UpForRetry => write!(f, "up_for_retry"),
            Self::Unknown => write!(f, "unknown"),
        }
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

impl fmt::Display for RunType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scheduled => write!(f, "scheduled"),
            Self::Manual => write!(f, "manual"),
            Self::Backfill => write!(f, "backfill"),
            Self::DatasetTriggered => write!(f, "dataset_triggered"),
            Self::AssetTriggered => write!(f, "asset_triggered"),
            Self::Unknown => write!(f, "unknown"),
        }
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
#[allow(clippy::struct_field_names)]
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

