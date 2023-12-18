use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};
use time::format_description;

use crate::app::state::App;

use super::TIME_FORMAT;

pub fn render_taskinstance_panel(f: &mut Frame, app: &mut App) {
    let rects = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(0)
        .split(f.size());

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Blue);

    let headers = ["Task ID", "Execution Date", "Duration", "State", "Tries"];
    let header_cells = headers
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let rows = app.taskinstances.items.iter().map(|item| {
        Row::new(vec![
            Line::from(item.task_id.as_str()),
            Line::from(if let Some(date) = item.execution_date {
                date.format(&format_description::parse(TIME_FORMAT).unwrap())
                    .unwrap()
                    .to_string()
            } else {
                "None".to_string()
            }),
            Line::from(if let Some(i) = item.duration {
                format!("{i}")
            } else {
                "None".to_string()
            }),
            Line::from(match item.state.as_str() {
                "success" => Span::styled(item.state.as_str(), Style::default().fg(Color::Green)),
                "running" => {
                    Span::styled(item.state.as_str(), Style::default().fg(Color::LightGreen))
                }
                "failed" => Span::styled(item.state.as_str(), Style::default().fg(Color::Red)),
                "queued" => {
                    Span::styled(item.state.as_str(), Style::default().fg(Color::LightBlue))
                }
                "up_for_retry" => {
                    Span::styled(item.state.as_str(), Style::default().fg(Color::LightYellow))
                }
                "upstream_failed" => Span::styled(
                    item.state.as_str(),
                    Style::default().fg(Color::Rgb(255, 165, 0)), // orange
                ),
                _ => Span::styled(item.state.as_str(), Style::default().fg(Color::White)),
            }),
            Line::from(format!("{:?}", item.try_number)),
        ])
        .bottom_margin(1)
    });
    let t = Table::new(
        rows,
        &[
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Length(20),
            Constraint::Length(15),
            Constraint::Length(5),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("TaskInstances"),
    )
    .highlight_style(selected_style)
    .highlight_symbol(">> ");

    f.render_stateful_widget(t, rects[0], &mut app.taskinstances.state);
}
