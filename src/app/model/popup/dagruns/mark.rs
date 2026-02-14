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
pub struct MarkDagRunPopup {
    pub dag_id: String,
    pub status: MarkState,
    pub confirm: bool,
    pub marked: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Display)]
pub enum MarkState {
    #[strum(to_string = "success")]
    Success,
    #[strum(to_string = "failed")]
    Failed,
    #[strum(to_string = "queued")]
    Queued,
}

impl MarkDagRunPopup {
    pub const fn new(marked: Vec<String>, dag_id: String) -> Self {
        Self {
            dag_id,
            status: MarkState::Success,
            confirm: false,
            marked,
        }
    }

    pub const fn next_state(&mut self) {
        self.status = match self.status {
            MarkState::Success => MarkState::Failed,
            MarkState::Failed => MarkState::Queued,
            MarkState::Queued => MarkState::Success,
        };
    }

    pub const fn previous_state(&mut self) {
        self.status = match self.status {
            MarkState::Success => MarkState::Queued,
            MarkState::Failed => MarkState::Success,
            MarkState::Queued => MarkState::Failed,
        };
    }
}

impl Model for MarkDagRunPopup {
    fn update(
        &mut self,
        event: &FlowrsEvent,
        _ctx: &crate::app::state::NavigationContext,
    ) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        if let FlowrsEvent::Key(key_event) = event {
            match key_event.code {
                KeyCode::Enter => {
                    // On Enter, we always return the key event, so the parent can close the popup
                    return (
                        Some(FlowrsEvent::Key(*key_event)),
                        self.marked
                            .iter()
                            .map(|i| WorkerMessage::MarkDagRun {
                                dag_run_id: i.clone(),
                                dag_id: self.dag_id.clone(),
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

impl Widget for &mut MarkDagRunPopup {
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

        let [_, success, _, failed, _, queued, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(11),
            Constraint::Length(2),
            Constraint::Length(10),
            Constraint::Length(2),
            Constraint::Length(10),
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

        // Queued button
        let (queued_style, queued_border) = if self.status == MarkState::Queued {
            (BUTTON_SELECTED, BORDER_SELECTED)
        } else {
            (BUTTON_DEFAULT, BORDER_DEFAULT)
        };
        let queued_btn = Paragraph::new("Queued")
            .style(queued_style)
            .centered()
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .border_style(queued_style.fg(queued_border)),
            );

        Clear.render(area, buffer);
        popup_block.render(area, buffer);
        text.render(header, buffer);
        success_btn.render(success, buffer);
        failed_btn.render(failed, buffer);
        queued_btn.render(queued, buffer);
    }
}
