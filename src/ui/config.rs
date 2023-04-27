use ratatui::{
    backend::Backend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use crate::app::state::App;

pub fn render_config_panel<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let rects = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(0)
        .split(f.size());

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Blue);

    let headers = ["Name", "Endpoint"];
    let header_cells = headers
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));

    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let rows = app.configs.items.iter().map(|item| {
        Row::new(vec![
            Spans::from(item.name.as_str()),
            Spans::from(item.endpoint.as_str()),
        ])
        // .height(height as u16)
        .bottom_margin(1)
    });

    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Config"))
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(50),
            Constraint::Length(30),
            Constraint::Min(10),
        ]);
    f.render_stateful_widget(t, rects[0], &mut app.configs.state);
}
