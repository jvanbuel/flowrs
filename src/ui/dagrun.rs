use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};
use time::format_description;

use crate::app::state::App;

use super::{constants::DEFAULT_STYLE, TIME_FORMAT};

pub fn render_dagrun_panel(f: &mut Frame, app: &mut App) {
    let rects = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(0)
        .split(f.size());

    let normal_style = DEFAULT_STYLE;

    let headers = ["DAG Id", "DAGRun Id", "Logical Date", "Type", "State"];
    let header_cells = headers.iter().map(|h| Cell::from(*h).style(normal_style));
    let header =
        Row::new(header_cells).style(DEFAULT_STYLE.reversed().add_modifier(Modifier::BOLD));

    let rows = app.dagruns.items.iter().map(|item| {
        Row::new(vec![
            Line::from(Span::styled(
                item.dag_id.as_str(),
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(item.dag_run_id.as_str()),
            Line::from(if let Some(date) = item.logical_date {
                date.format(&format_description::parse(TIME_FORMAT).unwrap())
                    .unwrap()
                    .to_string()
            } else {
                "None".to_string()
            }),
            Line::from(item.run_type.as_str()),
            Line::from(match item.state.as_str() {
                "success" => Span::styled("■", Style::default().fg(Color::Rgb(0, 128, 0))),
                "running" => Span::styled("■", DEFAULT_STYLE.fg(Color::LightGreen)),
                "failed" => Span::styled("■", DEFAULT_STYLE.fg(Color::Red)),
                "queued" => Span::styled("■", DEFAULT_STYLE.fg(Color::LightBlue)),
                _ => Span::styled("■", DEFAULT_STYLE.fg(Color::White)),
            }),
        ])
    });
    let t = Table::new(
        rows,
        &[
            Constraint::Min(20),
            Constraint::Percentage(15),
            Constraint::Min(22),
            Constraint::Length(20),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("DAGRuns")
            .style(DEFAULT_STYLE),
    )
    .style(DEFAULT_STYLE)
    .highlight_style(DEFAULT_STYLE.reversed());
    f.render_stateful_widget(t, rects[0], &mut app.dagruns.state);
}
