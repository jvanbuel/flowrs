use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget},
};

use crate::app::{
    events::custom::FlowrsEvent,
    model::{
        popup::{popup_area, render_yes_no, SelectedButton},
        Model,
    },
    worker::WorkerMessage,
};
use crate::ui::theme::theme;

use crate::airflow::model::common::{DagId, DagRunId};

#[derive(Debug)]
pub struct ClearDagRunPopup {
    pub dag_run_ids: Vec<DagRunId>,
    pub dag_id: DagId,
    pub(crate) selected_button: SelectedButton,
}

impl ClearDagRunPopup {
    pub fn new(dag_run_ids: Vec<DagRunId>, dag_id: DagId) -> Self {
        Self {
            dag_run_ids,
            dag_id,
            selected_button: SelectedButton::default(),
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
                    // If Yes is selected, we also return WorkerMessages to clear the dag runs
                    if self.selected_button.is_yes() {
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
                    self.selected_button.toggle();
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
        let t = theme();
        // Smaller popup: 40% width, auto height
        let area = popup_area(area, 40, 30);

        let popup_block = Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(t.border_style)
            .style(t.default_style);

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
        let text = Paragraph::new(message).style(t.default_style).centered();

        Clear.render(area, buffer);
        popup_block.render(area, buffer);
        text.render(header, buffer);
        render_yes_no(options, buffer, self.selected_button.is_yes(), true);
    }
}
