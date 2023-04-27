use ratatui::{
    backend::Backend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use crate::app::state::App;

pub fn render_dagrun_panel<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let rects = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(0)
        .split(f.size());

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Blue);

    let headers = ["DAG Id", "DAGRun Id", "Logical Date", "Type"];
    let header_cells = headers
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let rows = app.dagruns.items.iter().map(|item| {
        Row::new(vec![
            Spans::from(Span::styled(
                item.dag_id.as_str(),
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Spans::from(item.dag_run_id.as_str()),
            Spans::from(if let Some(date) = item.logical_date {
                date.to_string()
            } else {
                "None".to_string()
            }),
            Spans::from(item.run_type.as_str()),
        ])
        .bottom_margin(1)
    });
    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("DAGRuns"))
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Min(22),
            Constraint::Length(20),
        ]);
    f.render_stateful_widget(t, rects[0], &mut app.dagruns.state);
}
