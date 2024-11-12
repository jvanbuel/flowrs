use std::vec;

use crossterm::event::KeyCode;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Modifier, Stylize};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap};
use strum::Display;

use crate::app::events::custom::FlowrsEvent;
use crate::app::worker::WorkerMessage;
use crate::ui::constants::DEFAULT_STYLE;

use super::Model;

pub struct ClearDagRunPopup {
    pub dag_run_id: String,
    pub dag_id: String,
    pub confirm: bool,
}

impl ClearDagRunPopup {
    pub fn new(dag_run_id: String, dag_id: String) -> Self {
        ClearDagRunPopup {
            dag_run_id,
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
                    // If the confirm flag is set, we also return a WorkerMessage to clear the dag run
                    if self.confirm {
                        return (
                            Some(FlowrsEvent::Key(*key_event)),
                            vec![WorkerMessage::ClearDagRun {
                                dag_run_id: self.dag_run_id.clone(),
                                dag_id: self.dag_id.clone(),
                            }],
                        );
                    } else {
                        return (Some(FlowrsEvent::Key(*key_event)), vec![]);
                    }
                }
                KeyCode::Char('j')
                | KeyCode::Down
                | KeyCode::Char('k')
                | KeyCode::Up
                | KeyCode::Char('h')
                | KeyCode::Left
                | KeyCode::Char('l')
                | KeyCode::Right => {
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
            .borders(Borders::ALL)
            .title("Clear DAG Run - press <Enter> to confirm, <q>|<Esc> to close")
            .border_style(DEFAULT_STYLE)
            .style(DEFAULT_STYLE)
            .title_style(DEFAULT_STYLE.add_modifier(Modifier::BOLD));

        let text = Paragraph::new("Are you sure you want to clear this DAG Run?")
            .style(DEFAULT_STYLE)
            .block(Block::default())
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
            .block(Block::default().borders(Borders::ALL));

        let no_text = Paragraph::new("No")
            .style(if !self.confirm {
                DEFAULT_STYLE.reversed()
            } else {
                DEFAULT_STYLE
            })
            .centered()
            .block(Block::default().borders(Borders::ALL));

        Clear.render(area, buffer); //this clears out the background
        popup_block.render(area, buffer);
        text.render(header, buffer);
        yes_text.render(yes, buffer);
        no_text.render(no, buffer);
    }
}

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
                                dag_run_id: i.to_string(),
                                dag_id: self.dag_id.to_string(),
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
            .borders(Borders::ALL)
            .title("Mark DAG Run - press <Enter> to confirm, <q>|<Esc> to close")
            .border_style(DEFAULT_STYLE)
            .style(DEFAULT_STYLE)
            .title_style(DEFAULT_STYLE.add_modifier(Modifier::BOLD));

        let text = Paragraph::new("Select the status to mark this DAG Run with:")
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

        let queued_text = Paragraph::new("Queued")
            .style(if self.status == MarkState::Queued {
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

/// helper function to create a centered rect using up certain percentage of the available rect `r`
#[allow(dead_code)]
pub fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
