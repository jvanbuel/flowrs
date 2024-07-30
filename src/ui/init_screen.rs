use crate::{app::state::App, ascii_flowrs::ASCII_LOGO};
use ansi_to_tui::IntoText;
use indoc::indoc;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Stylize},
    text::{self, Line, Text},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Wrap},
    Frame,
};

use super::constants::DEFAULT_STYLE;

pub fn render_init_screen(f: &mut Frame, app: &mut App) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Length(10000)])
        .split(f.size());

    let text = ASCII_LOGO.into_text().unwrap();
    let paragraph = Paragraph::new(text);

    f.render_widget(paragraph, layout[0])
}
