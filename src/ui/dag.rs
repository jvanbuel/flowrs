use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};
use time::format_description;

use crate::app::state::App;
use crate::ui::TIME_FORMAT;

pub fn render_dag_panel(f: &mut Frame, app: &mut App) {
    let rects = if app.filter.is_enabled() {
        let rects = Layout::default()
            .constraints(if app.filter.is_enabled() {
                [Constraint::Percentage(90), Constraint::Percentage(10)].as_ref()
            } else {
                [Constraint::Percentage(100)].as_ref()
            })
            .margin(0)
            .split(f.size());

        let filter = app.filter.prefix().clone();

        let paragraph = Paragraph::new(filter.unwrap_or("".to_string()))
            .block(Block::default().borders(Borders::ALL).title("filter"));
        f.render_widget(paragraph, rects[1]);

        rects
    } else {
        Layout::default()
            .constraints(if app.filter.is_enabled() {
                [Constraint::Percentage(90), Constraint::Percentage(10)].as_ref()
            } else {
                [Constraint::Percentage(100)].as_ref()
            })
            .margin(0)
            .split(f.size())
    };

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Blue);

    let headers = ["Active", "Name", "Owners", "Schedule", "Next Run"];
    let header_cells = headers
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let rows = app.filtered_dags.items.iter().map(|item| {
        Row::new(vec![
            if item.is_paused {
                Line::from(Span::styled("🔘", Style::default().fg(Color::Gray)))
            } else {
                Line::from(Span::styled("🔵", Style::default().fg(Color::Gray)))
            },
            Line::from(Span::styled(
                item.dag_id.as_str(),
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(item.owners.join(", ")),
            if let Some(schedule) = &item.schedule_interval {
                Line::from(Span::styled(
                    schedule.value.clone().unwrap_or_else(|| "None".to_string()),
                    Style::default().fg(Color::LightYellow),
                ))
            } else {
                Line::from(Span::styled(
                    "None",
                    Style::default().fg(Color::LightYellow),
                ))
            },
            Line::from(if let Some(date) = item.next_dagrun {
                date.format(&format_description::parse(TIME_FORMAT).unwrap())
                    .unwrap()
                    .to_string()
            } else {
                "None".to_string()
            }),
        ])
        // .height(height as u16)
        .bottom_margin(1)
    });
    let t = Table::new(
        rows,
        &[
            Constraint::Length(7),
            Constraint::Percentage(20),
            Constraint::Min(15),
            Constraint::Length(10),
            Constraint::Length(30),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title("DAGs"))
    .highlight_style(selected_style)
    .highlight_symbol(">> ");
    f.render_stateful_widget(t, rects[0], &mut app.filtered_dags.state);
}
