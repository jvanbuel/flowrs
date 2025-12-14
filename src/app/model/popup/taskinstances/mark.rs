use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget},
};
use strum::Display;

use crate::{
    app::{
        events::custom::FlowrsEvent,
        model::{popup::popup_area, Model},
        worker::WorkerMessage,
    },
    ui::theme::{
        BORDER_DEFAULT, BORDER_SELECTED, BUTTON_DEFAULT, BUTTON_SELECTED, DEFAULT_STYLE,
        SURFACE_STYLE,
    },
};
pub struct MarkTaskInstancePopup {
    pub dag_id: String,
    pub dag_run_id: String,
    pub status: MarkState,
    pub confirm: bool,
    pub marked: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Display)]
pub enum MarkState {
    #[strum(to_string = "success")]
    Success,
    #[strum(to_string = "failed")]
    Failed,
    #[strum(to_string = "skipped")]
    Skipped,
}

impl MarkTaskInstancePopup {
    pub fn new(marked: Vec<String>, dag_id: &str, dag_run_id: &str) -> Self {
        MarkTaskInstancePopup {
            dag_id: dag_id.to_string(),
            status: MarkState::Success,
            confirm: false,
            marked,
            dag_run_id: dag_run_id.to_string(),
        }
    }

    pub fn next_state(&mut self) {
        self.status = match self.status {
            MarkState::Success => MarkState::Failed,
            MarkState::Failed => MarkState::Skipped,
            MarkState::Skipped => MarkState::Success,
        };
    }

    pub fn previous_state(&mut self) {
        self.status = match self.status {
            MarkState::Success => MarkState::Skipped,
            MarkState::Failed => MarkState::Success,
            MarkState::Skipped => MarkState::Failed,
        };
    }
}

impl Model for MarkTaskInstancePopup {
    fn update(&mut self, event: &FlowrsEvent) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        if let FlowrsEvent::Key(key_event) = event {
            match key_event.code {
                KeyCode::Enter => {
                    // On Enter, we always return the key event, so the parent can close the popup
                    return (
                        Some(FlowrsEvent::Key(*key_event)),
                        self.marked
                            .iter()
                            .map(|i| WorkerMessage::MarkTaskInstance {
                                dag_run_id: self.dag_run_id.clone(),
                                dag_id: self.dag_id.clone(),
                                task_id: i.clone(),
                                status: self.status.clone(),
                            })
                            .collect(),
                    );
                }
                KeyCode::Char('j' | 'h') | KeyCode::Down | KeyCode::Left => {
                    self.previous_state();
                    return (None, vec![]);
                }
                KeyCode::Char('k' | 'l') | KeyCode::Up | KeyCode::Right => {
                    self.next_state();
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

impl Widget for &mut MarkTaskInstancePopup {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        // Smaller popup: 35% width, auto height
        let area = popup_area(area, 35, 30);

        let [_, header, options, _] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .flex(Flex::Center)
        .areas(area);

        let popup_block = Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(DEFAULT_STYLE)
            .style(SURFACE_STYLE);

        let text = Paragraph::new("Mark status as")
            .style(DEFAULT_STYLE)
            .centered();

        let [_, success, _, failed, _, skipped, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(11),
            Constraint::Length(2),
            Constraint::Length(10),
            Constraint::Length(2),
            Constraint::Length(11),
            Constraint::Fill(1),
        ])
        .areas(options);

        // Success button
        let (success_style, success_border) = if self.status == MarkState::Success {
            (BUTTON_SELECTED, BORDER_SELECTED)
        } else {
            (BUTTON_DEFAULT, BORDER_DEFAULT)
        };
        let success_btn = Paragraph::new("Success")
            .style(success_style)
            .centered()
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .border_style(success_style.fg(success_border)),
            );

        // Failed button
        let (failed_style, failed_border) = if self.status == MarkState::Failed {
            (BUTTON_SELECTED, BORDER_SELECTED)
        } else {
            (BUTTON_DEFAULT, BORDER_DEFAULT)
        };
        let failed_btn = Paragraph::new("Failed")
            .style(failed_style)
            .centered()
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .border_style(failed_style.fg(failed_border)),
            );

        // Skipped button
        let (skipped_style, skipped_border) = if self.status == MarkState::Skipped {
            (BUTTON_SELECTED, BORDER_SELECTED)
        } else {
            (BUTTON_DEFAULT, BORDER_DEFAULT)
        };
        let skipped_btn = Paragraph::new("Skipped")
            .style(skipped_style)
            .centered()
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .border_style(skipped_style.fg(skipped_border)),
            );

        Clear.render(area, buffer);
        popup_block.render(area, buffer);
        text.render(header, buffer);
        success_btn.render(success, buffer);
        failed_btn.render(failed, buffer);
        skipped_btn.render(skipped, buffer);
    }
}
