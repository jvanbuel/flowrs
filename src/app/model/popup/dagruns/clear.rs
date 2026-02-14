use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget},
};

use crate::{
    app::{
        events::custom::FlowrsEvent,
        model::{
            popup::{popup_area, themed_button},
            Model,
        },
        worker::WorkerMessage,
    },
    ui::theme::{BORDER_STYLE, DEFAULT_STYLE, SURFACE_STYLE},
};

pub struct ClearDagRunPopup {
    pub dag_run_ids: Vec<String>,
    pub dag_id: String,
    pub confirm: bool,
}

impl ClearDagRunPopup {
    pub const fn new(dag_run_ids: Vec<String>, dag_id: String) -> Self {
        Self {
            dag_run_ids,
            dag_id,
            confirm: false,
        }
    }
}

impl Model for ClearDagRunPopup {
    fn update(
        &mut self,
        event: &FlowrsEvent,
        _ctx: &crate::app::state::NavigationContext,
    ) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
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
                KeyCode::Char('j' | 'k' | 'h' | 'l')
                | KeyCode::Down
                | KeyCode::Up
                | KeyCode::Left
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
        // Smaller popup: 40% width, auto height
        let area = popup_area(area, 40, 30);

        let popup_block = Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(BORDER_STYLE)
            .style(SURFACE_STYLE);

        // Use inner area for content layout to avoid overlapping the border
        let inner = popup_block.inner(area);

        let [_, header, options, _] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .flex(Flex::Center)
        .areas(inner);

        let message = if self.dag_run_ids.len() == 1 {
            "Clear this DAG Run?".to_string()
        } else {
            format!("Clear {} DAG Runs?", self.dag_run_ids.len())
        };
        let text = Paragraph::new(message).style(DEFAULT_STYLE).centered();

        let [_, yes, _, no, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(8),
            Constraint::Length(2),
            Constraint::Length(8),
            Constraint::Fill(1),
        ])
        .areas(options);

        let yes_btn = themed_button("Yes", self.confirm);
        let no_btn = themed_button("No", !self.confirm);

        Clear.render(area, buffer);
        popup_block.render(area, buffer);
        text.render(header, buffer);
        yes_btn.render(yes, buffer);
        no_btn.render(no, buffer);
    }
}
