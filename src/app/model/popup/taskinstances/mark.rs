use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Modifier, Stylize},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};
use strum::Display;

use crate::{
    app::{
        events::custom::FlowrsEvent,
        model::{popup::popup_area, Model},
        worker::WorkerMessage,
    },
    ui::constants::DEFAULT_STYLE,
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
                                dag_run_id: i.to_string(),
                                dag_id: self.dag_id.to_string(),
                                task_id: i.to_string(),
                                status: self.status.clone(),
                            })
                            .collect(),
                    );
                }
                KeyCode::Char('j') | KeyCode::Down | KeyCode::Char('h') | KeyCode::Left => {
                    self.previous_state();
                    return (None, vec![]);
                }
                KeyCode::Char('k') | KeyCode::Up | KeyCode::Char('l') | KeyCode::Right => {
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
            .borders(Borders::ALL)
            .title("Mark Task Instance - press <Enter> to confirm, <q>|<Esc> to close")
            .border_style(DEFAULT_STYLE)
            .style(DEFAULT_STYLE)
            .title_style(DEFAULT_STYLE.add_modifier(Modifier::BOLD));

        let text = Paragraph::new("Select the status to mark this TaskInstance with:")
            .style(DEFAULT_STYLE)
            .block(Block::default())
            .centered()
            .wrap(Wrap { trim: true });

        let [_, success, _, failed, _, queued, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(10),
            Constraint::Percentage(5),
            Constraint::Length(10),
            Constraint::Percentage(5),
            Constraint::Length(10),
            Constraint::Fill(1),
        ])
        .areas(options);

        let success_text = Paragraph::new("Success")
            .style(if self.status == MarkState::Success {
                DEFAULT_STYLE.reversed()
            } else {
                DEFAULT_STYLE
            })
            .centered()
            .block(Block::default().borders(Borders::ALL));

        let failed_text = Paragraph::new("Failed")
            .style(if self.status == MarkState::Failed {
                DEFAULT_STYLE.reversed()
            } else {
                DEFAULT_STYLE
            })
            .centered()
            .block(Block::default().borders(Borders::ALL));

        let queued_text = Paragraph::new("Skipped")
            .style(if self.status == MarkState::Skipped {
                DEFAULT_STYLE.reversed()
            } else {
                DEFAULT_STYLE
            })
            .centered()
            .block(Block::default().borders(Borders::ALL));

        Clear.render(area, buffer); //this clears out the background
        popup_block.render(area, buffer);
        text.render(header, buffer);
        success_text.render(success, buffer);
        failed_text.render(failed, buffer);
        queued_text.render(queued, buffer);
    }
}
