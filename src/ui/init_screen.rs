use crate::ui::constants::ASCII_LOGO;
use ansi_to_tui::IntoText;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::Paragraph,
    Frame,
};

pub fn render_init_screen(f: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Length(10000)])
        .split(f.area());

    let text = ASCII_LOGO.into_text().unwrap();
    let paragraph = Paragraph::new(text);

    f.render_widget(paragraph, layout[0])
}
