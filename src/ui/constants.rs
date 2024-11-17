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
