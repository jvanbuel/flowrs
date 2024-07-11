use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style, Styled, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};
use time::format_description;

use crate::app::state::App;
use crate::ui::TIME_FORMAT;

use super::constants::DEFAULT_STYLE;

pub fn render_dag_panel(f: &mut Frame, app: &mut App) {
    let rects = if app.filter.is_enabled() {
        let rects = Layout::default()
            .constraints([Constraint::Fill(90), Constraint::Max(3)].as_ref())
            .margin(0)
            .split(f.size());

        let filter = app.filter.prefix().clone();

        let paragraph = Paragraph::new(filter.unwrap_or("".to_string()))
            .block(Block::default().borders(Borders::ALL).title("filter"))
            .set_style(DEFAULT_STYLE);
        f.render_widget(paragraph, rects[1]);

        rects
    } else {
        Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .margin(0)
            .split(f.size())
    };

    if app.is_loading {
        let text = "Loading...";
        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("DAGs"))
            .style(Style::default().fg(Color::LightYellow));
        f.render_widget(paragraph, rects[0]);
        return;
    }

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);

    let headers = ["Active", "Name", "Owners", "Schedule", "Next Run"];
    let header_cells = headers.iter().map(|h| Cell::from(*h));
    let header = Row::new(header_cells)
        .style(DEFAULT_STYLE.reversed())
        .add_modifier(Modifier::BOLD);
    // .underlined();
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
        // .bottom_margin(1)
        .style(DEFAULT_STYLE)
    });
    let t = Table::new(
        rows,
        &[
            Constraint::Length(7),
            Constraint::Percentage(40),
            Constraint::Max(15),
            Constraint::Length(10),
            Constraint::Fill(1),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("DAGs")
            .border_style(DEFAULT_STYLE)
            .style(DEFAULT_STYLE),
    )
    .highlight_style(selected_style);
    f.render_stateful_widget(t, rects[0], &mut app.filtered_dags.state);
}
