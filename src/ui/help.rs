use ratatui::{
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use crate::app::state::App;

use super::constants::DM_RGB;

pub fn render_help_panel(f: &mut Frame, app: &mut App) {
    let rects = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(0)
        .split(f.size());

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(DM_RGB);

    let headers = ["Name", "Endpoint"];
    let header_cells = headers.iter().map(|h| Cell::from(*h).style(normal_style));

    let header = Row::new(header_cells)
        .style(normal_style.add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);
    let rows = app.configs.items.iter().map(|item| {
        Row::new(vec![
            Line::from(item.name.as_str()),
            Line::from(item.endpoint.as_str()),
        ])
        .bottom_margin(1)
    });

    let t = Table::new(
        rows,
        &[
            Constraint::Percentage(50),
            Constraint::Length(30),
            Constraint::Min(10),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title("Help"))
    .highlight_style(selected_style)
    .highlight_symbol(">> ");

    f.render_widget(t, rects[0])
}
