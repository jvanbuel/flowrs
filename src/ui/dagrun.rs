use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};
use time::format_description;

use crate::app::state::App;

use super::{constants::DM_RGB, TIME_FORMAT};

pub fn render_dagrun_panel(f: &mut Frame, app: &mut App) {
    let rects = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(0)
        .split(f.size());

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(DM_RGB);

    let headers = ["DAG Id", "DAGRun Id", "Logical Date", "Type", "State"];
    let header_cells = headers.iter().map(|h| Cell::from(*h).style(normal_style));
    let header = Row::new(header_cells)
        .style(normal_style.add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);
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
                "success" => Span::styled(
                    "■",
                    Style::default()
                        .fg(Color::Rgb(0, 128, 0))
                        .add_modifier(Modifier::BOLD),
                ),
                "running" => Span::styled("■", Style::default().fg(Color::LightGreen)),
                "failed" => Span::styled("■", Style::default().fg(Color::Red)),
                "queued" => Span::styled("■", Style::default().fg(Color::LightBlue)),
                _ => Span::styled("■", Style::default().fg(Color::White)),
            }),
        ])
        .bottom_margin(1)
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
    .block(Block::default().borders(Borders::ALL).title("DAGRuns"))
    .style(normal_style)
    .highlight_style(selected_style);
    f.render_stateful_widget(t, rects[0], &mut app.dagruns.state);
}
