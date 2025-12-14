use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Modifier, Stylize},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget, Wrap},
};

use crate::{
    app::{
        events::custom::FlowrsEvent,
        model::{popup::popup_area, Model},
        worker::WorkerMessage,
    },
    ui::constants::DEFAULT_STYLE,
};

pub struct ClearDagRunPopup {
    pub dag_run_ids: Vec<String>,
    pub dag_id: String,
    pub confirm: bool,
}

impl ClearDagRunPopup {
    pub fn new(dag_run_ids: Vec<String>, dag_id: String) -> Self {
        ClearDagRunPopup {
            dag_run_ids,
            dag_id,
            confirm: false,
        }
    }
}

impl Model for ClearDagRunPopup {
    fn update(&mut self, event: &FlowrsEvent) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        if let FlowrsEvent::Key(key_event) = event {
            match key_event.code {
                KeyCode::Enter => {
                    // On Enter, we always return the key event, so the parent can close the popup
                    // If the confirm flag is set, we also return WorkerMessages to clear the dag runs
                    if self.confirm {
                        return (
                            Some(FlowrsEvent::Key(*key_event)),
                            self.dag_run_ids
                                .iter()
                                .map(|dag_run_id| WorkerMessage::ClearDagRun {
                                    dag_run_id: dag_run_id.clone(),
                                    dag_id: self.dag_id.clone(),
                                })
                                .collect(),
                        );
                    }
                    return (Some(FlowrsEvent::Key(*key_event)), vec![]);
                }
                KeyCode::Char('j' | 'k' | 'h' | 'l') | KeyCode::Down | KeyCode::Up |
KeyCode::Left | KeyCode::Right => {
                    // For any movement vim key, we toggle the confirm flag, and we consume the event
                    self.confirm = !self.confirm;
                    return (None, vec![]);
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    // On Esc, we always return the key event, so the parent can close the popup, without clearing the dag run
                    return (Some(FlowrsEvent::Key(*key_event)), vec![]);
                }
                _ => {}
            }
        }
        (Some(event.clone()), vec![])
    }
}

impl Widget for &mut ClearDagRunPopup {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let area = popup_area(area, 50, 50);

        let [_, header, options, _] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .flex(Flex::Center)
        .areas(area);

        let popup_block = Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .title("Clear DAG Run")
            .border_style(DEFAULT_STYLE)
            .style(DEFAULT_STYLE)
            .title_style(DEFAULT_STYLE.add_modifier(Modifier::BOLD));

        let message = if self.dag_run_ids.len() == 1 {
            "Are you sure you want to clear this DAG Run?".to_string()
        } else {
            format!(
                "Are you sure you want to clear {} DAG Runs?",
                self.dag_run_ids.len()
            )
        };
        let text = Paragraph::new(message)
            .style(DEFAULT_STYLE)
            .block(Block::default().border_type(BorderType::Rounded))
            .centered()
            .wrap(Wrap { trim: true });

        let [_, yes, _, no, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(7),
            Constraint::Percentage(5),
            Constraint::Length(7),
            Constraint::Fill(1),
        ])
        .areas(options);

        let yes_text = Paragraph::new("Yes")
            .style(if self.confirm {
                DEFAULT_STYLE.reversed()
            } else {
                DEFAULT_STYLE
            })
            .centered()
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL),
            );

        let no_text = Paragraph::new("No")
            .style(if self.confirm {
                DEFAULT_STYLE
            } else {
                DEFAULT_STYLE.reversed()
            })
            .centered()
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL),
            );

        Clear.render(area, buffer); //this clears out the background
        popup_block.render(area, buffer);
        text.render(header, buffer);
        yes_text.render(yes, buffer);
        no_text.render(no, buffer);
    }
}
