use ratatui::style::{Color, Modifier, Style};

pub const DM_RGB: Color = Color::Rgb(192, 175, 226);

pub const DEFAULT_STYLE: Style = Style {
    fg: Some(DM_RGB),
    bg: Some(Color::Black),
    underline_color: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

pub const ALTERNATING_ROW_COLOR: Color = Color::Rgb(33, 34, 35);
pub const MARKED_COLOR: Color = Color::Rgb(255, 255, 224);

pub const ASCII_LOGO: &str = include_str!("logo/logo.ascii");

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
        match state {
            AirflowStateColor::Success => Color::Rgb(0, 128, 0),
            AirflowStateColor::Failed => Color::Rgb(255, 0, 0),
            AirflowStateColor::Running => Color::Rgb(34, 255, 34),
            AirflowStateColor::Queued => Color::Rgb(128, 128, 128),
            AirflowStateColor::UpForRetry => Color::Rgb(255, 215, 0),
            AirflowStateColor::UpForReschedule => Color::Rgb(111, 231, 219),
            AirflowStateColor::Skipped => Color::Rgb(255, 142, 198),
            AirflowStateColor::UpstreamFailed => Color::Rgb(255, 165, 0),
            AirflowStateColor::None => Color::Reset,
        }
    }
}
