use ratatui::style::Color;

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
            AirflowStateColor::None => Color::Reset,
        }
    }
}
