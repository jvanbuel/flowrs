use crate::ui::constants::ROTATING_LOGO;
use ansi_to_tui::IntoText;
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    Frame,
};

pub fn render_init_screen(f: &mut Frame, index: u32) {
    // let text = ASCII_LOGO.into_text().unwrap();
    let text = ROTATING_LOGO[index as usize % ROTATING_LOGO.len()]
        .into_text()
        .expect("ROTATING_LOGO should contain valid ANSI text");

    let area = center(
        f.area(),
        #[allow(clippy::cast_possible_truncation)]
        Constraint::Length(text.width() as u16),
        #[allow(clippy::cast_possible_truncation)]
        Constraint::Length(text.height() as u16),
    );

    f.render_widget(text, area);
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}
