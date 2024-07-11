use ratatui::{
    layout::{Constraint, Layout},
    style::{Modifier, Stylize},
    text::Line,
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use crate::app::state::App;

use super::constants::DEFAULT_STYLE;

pub fn render_config_panel(f: &mut Frame, app: &mut App) {
    let rects = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(0)
        .split(f.size());

    let selected_style = DEFAULT_STYLE.add_modifier(Modifier::REVERSED);

    let headers = ["Name", "Endpoint"];
    let header_cells = headers.iter().map(|h| Cell::from(*h).style(DEFAULT_STYLE));

    let header =
        Row::new(header_cells).style(DEFAULT_STYLE.reversed().add_modifier(Modifier::BOLD));

    let rows = app.configs.items.iter().map(|item| {
        Row::new(vec![
            Line::from(item.name.as_str()),
            Line::from(item.endpoint.as_str()),
        ])
        // .height(height as u16)
    });

    let t = Table::new(
        rows,
        &[Constraint::Percentage(20), Constraint::Percentage(80)],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title("Config"))
    .style(DEFAULT_STYLE)
    .highlight_style(selected_style);
    f.render_stateful_widget(t, rects[0], &mut app.configs.state);
}
