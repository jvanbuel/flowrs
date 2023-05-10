use ratatui::{
    backend::Backend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use crate::app::state::App;

pub fn render_taskinstance_panel<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let rects = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(0)
        .split(f.size());

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Blue);

    let headers = ["Task ID", "Execution Date", "Duration", "State"];
    let header_cells = headers
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let rows = app.taskinstances.items.iter().map(|item| {
        Row::new(vec![
            Spans::from(item.task_id.as_str()),
            Spans::from(item.execution_date.to_string()),
            Spans::from(if let Some(i) = item.duration {
                format!("{i}")
            } else {
                "None".to_string()
            }),
            Spans::from(match item.state.as_str() {
                "success" => Span::styled(item.state.as_str(), Style::default().fg(Color::Green)),
                "running" => {
                    Span::styled(item.state.as_str(), Style::default().fg(Color::LightGreen))
                }
                "failed" => Span::styled(item.state.as_str(), Style::default().fg(Color::Red)),
                "queued" => {
                    Span::styled(item.state.as_str(), Style::default().fg(Color::LightBlue))
                }

                _ => Span::styled(item.state.as_str(), Style::default().fg(Color::White)),
            }),
        ])
        .bottom_margin(1)
    });
    let t = Table::new(rows)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("TaskInstances"),
        )
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Length(20),
            Constraint::Length(10),
        ]);
    f.render_stateful_widget(t, rects[0], &mut app.taskinstances.state);
}
