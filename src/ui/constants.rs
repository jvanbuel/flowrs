use ratatui::style::Color;

use crate::airflow::model::common::dagrun::DagRunState;
use crate::airflow::model::common::taskinstance::TaskInstanceState;

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
        use super::theme::theme;
        let t = theme();
        match state {
            AirflowStateColor::Success => t.state_success,
            AirflowStateColor::Failed => t.state_failed,
            AirflowStateColor::Running => t.state_running,
            AirflowStateColor::Queued => t.state_queued,
            AirflowStateColor::UpForRetry => t.state_up_for_retry,
            AirflowStateColor::UpForReschedule => t.state_up_for_reschedule,
            AirflowStateColor::Skipped => t.state_skipped,
            AirflowStateColor::UpstreamFailed => t.state_upstream_failed,
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
