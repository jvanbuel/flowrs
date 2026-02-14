use ratatui::style::Color;

use crate::airflow::model::common::dagrun::DagRunState;
use crate::airflow::model::common::taskinstance::TaskInstanceState;

// Re-export from theme for backward compatibility
pub use super::theme::DEFAULT_STYLE;

pub const ASCII_LOGO: &str = include_str!("logo/logo.ascii");

pub const ROTATING_LOGO: [&str; 16] = [
    include_str!("../../image/rotation/ascii/0.ascii"),
    include_str!("../../image/rotation/ascii/1.ascii"),
    include_str!("../../image/rotation/ascii/2.ascii"),
    include_str!("../../image/rotation/ascii/3.ascii"),
    include_str!("../../image/rotation/ascii/4.ascii"),
    include_str!("../../image/rotation/ascii/5.ascii"),
    include_str!("../../image/rotation/ascii/6.ascii"),
    include_str!("../../image/rotation/ascii/7.ascii"),
    include_str!("../../image/rotation/ascii/8.ascii"),
    include_str!("../../image/rotation/ascii/9.ascii"),
    include_str!("../../image/rotation/ascii/10.ascii"),
    include_str!("../../image/rotation/ascii/11.ascii"),
    include_str!("../../image/rotation/ascii/12.ascii"),
    include_str!("../../image/rotation/ascii/13.ascii"),
    include_str!("../../image/rotation/ascii/14.ascii"),
    include_str!("../../image/rotation/ascii/15.ascii"),
];

pub enum AirflowStateColor {
    Success,
    Failed,
    Running,
    Queued,
    UpForRetry,
    UpForReschedule,
    Skipped,
    UpstreamFailed,
    None,
}

impl From<AirflowStateColor> for Color {
    fn from(state: AirflowStateColor) -> Self {
        use super::theme;
        match state {
            AirflowStateColor::Success => theme::STATE_SUCCESS,
            AirflowStateColor::Failed => theme::STATE_FAILED,
            AirflowStateColor::Running => theme::STATE_RUNNING,
            AirflowStateColor::Queued => theme::STATE_QUEUED,
            AirflowStateColor::UpForRetry => theme::STATE_UP_FOR_RETRY,
            AirflowStateColor::UpForReschedule => theme::STATE_UP_FOR_RESCHEDULE,
            AirflowStateColor::Skipped => theme::STATE_SKIPPED,
            AirflowStateColor::UpstreamFailed => theme::STATE_UPSTREAM_FAILED,
            AirflowStateColor::None => Self::Reset,
        }
    }
}

impl From<&DagRunState> for AirflowStateColor {
    fn from(state: &DagRunState) -> Self {
        match state {
            DagRunState::Success => Self::Success,
            DagRunState::Running => Self::Running,
            DagRunState::Failed => Self::Failed,
            DagRunState::Queued => Self::Queued,
            DagRunState::UpForRetry => Self::UpForRetry,
            DagRunState::Unknown => Self::None,
        }
    }
}

impl From<&TaskInstanceState> for AirflowStateColor {
    fn from(state: &TaskInstanceState) -> Self {
        match state {
            TaskInstanceState::Success => Self::Success,
            TaskInstanceState::Running => Self::Running,
            TaskInstanceState::Failed => Self::Failed,
            TaskInstanceState::Queued => Self::Queued,
            TaskInstanceState::UpForRetry => Self::UpForRetry,
            TaskInstanceState::UpForReschedule => Self::UpForReschedule,
            TaskInstanceState::Skipped => Self::Skipped,
            TaskInstanceState::UpstreamFailed => Self::UpstreamFailed,
            _ => Self::None,
        }
    }
}
