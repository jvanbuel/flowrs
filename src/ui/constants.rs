use ratatui::style::{Color, Modifier, Style};

pub const DM_RGB: Color = Color::Rgb(192, 175, 226);

pub const DEFAULT_STYLE: Style = Style {
    fg: Some(DM_RGB),
    bg: Some(Color::Black),
    underline_color: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

pub const ASCII_LOGO: &str = include_str!("logo/logo.ascii");

