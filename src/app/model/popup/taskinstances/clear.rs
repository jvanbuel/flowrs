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

pub struct ClearTaskInstancePopup {
    pub dag_run_id: String,
    pub dag_id: String,
    pub task_ids: Vec<String>,
    pub confirm: bool,
}

impl ClearTaskInstancePopup {
    pub fn new(dag_run_id: &str, dag_id: &str, task_ids: Vec<String>) -> Self {
        Self {
            dag_run_id: dag_run_id.to_string(),
            dag_id: dag_id.to_string(),
            task_ids,
            confirm: false,
        }
    }
}

impl Model for ClearTaskInstancePopup {
    fn update(&mut self, event: &FlowrsEvent, _ctx: &crate::app::state::NavigationContext) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        if let FlowrsEvent::Key(key_event) = event {
            match key_event.code {
                KeyCode::Enter => {
                    // On Enter, we always return the key event, so the parent can close the popup
                    // If the confirm flag is set, we also return WorkerMessages to clear the task instances
                    if self.confirm {
                        return (
                            Some(FlowrsEvent::Key(*key_event)),
                            self.task_ids
                                .iter()
                                .map(|task_id| WorkerMessage::ClearTaskInstance {
                                    dag_run_id: self.dag_run_id.clone(),
                                    dag_id: self.dag_id.clone(),
                                    task_id: task_id.clone(),
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

impl Widget for &mut ClearTaskInstancePopup {
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

        let message = if self.task_ids.len() == 1 {
            "Clear this Task Instance?".to_string()
        } else {
            format!("Clear {} Task Instances?", self.task_ids.len())
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
