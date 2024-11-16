use crate::ui::constants::ASCII_LOGO;
use ansi_to_tui::IntoText;
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    Frame,
};

pub fn render_init_screen(f: &mut Frame) {
    let text = ASCII_LOGO.into_text().unwrap();
    // let paragraph = Paragraph::new(&text);

    let area = center(
        f.area(),
        Constraint::Length(text.width() as u16),
        Constraint::Length(text.height() as u16),
    );

    f.render_widget(text, area)
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}
