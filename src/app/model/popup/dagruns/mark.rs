use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Modifier, Stylize},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget, Wrap},
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
pub struct MarkDagRunPopup {
    pub dag_id: String,
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
    #[strum(to_string = "queued")]
    Queued,
}

impl MarkDagRunPopup {
    pub fn new(marked: Vec<String>, dag_id: String) -> Self {
        MarkDagRunPopup {
            dag_id,
            status: MarkState::Success,
            confirm: false,
            marked,
        }
    }

    pub fn next_state(&mut self) {
        self.status = match self.status {
            MarkState::Success => MarkState::Failed,
            MarkState::Failed => MarkState::Queued,
            MarkState::Queued => MarkState::Success,
        };
    }

    pub fn previous_state(&mut self) {
        self.status = match self.status {
            MarkState::Success => MarkState::Queued,
            MarkState::Failed => MarkState::Success,
            MarkState::Queued => MarkState::Failed,
        };
    }
}

impl Model for MarkDagRunPopup {
    fn update(&mut self, event: &FlowrsEvent) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        if let FlowrsEvent::Key(key_event) = event {
            match key_event.code {
                KeyCode::Enter => {
                    // On Enter, we always return the key event, so the parent can close the popup
                    return (
                        Some(FlowrsEvent::Key(*key_event)),
                        // vec![WorkerMessage::MarkDagRun {
                        //     dag_run_id: self.dag_run_id.clone(),
                        //     dag_id: self.dag_id.clone(),
                        //     status: self.status.clone(),
                        // }],
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
            .title("Mark DAG Run")
            .border_style(DEFAULT_STYLE)
            .style(DEFAULT_STYLE)
            .title_style(DEFAULT_STYLE.add_modifier(Modifier::BOLD));

        let text = Paragraph::new("Select the status to mark this DAG Run with:")
            .style(DEFAULT_STYLE)
            .block(Block::default().border_type(BorderType::Rounded))
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
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL),
            );

        let failed_text = Paragraph::new("Failed")
            .style(if self.status == MarkState::Failed {
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

        let queued_text = Paragraph::new("Queued")
            .style(if self.status == MarkState::Queued {
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

        Clear.render(area, buffer); //this clears out the background
        popup_block.render(area, buffer);
        text.render(header, buffer);
        success_text.render(success, buffer);
        failed_text.render(failed, buffer);
        queued_text.render(queued, buffer);
    }
}
