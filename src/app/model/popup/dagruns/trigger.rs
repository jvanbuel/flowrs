use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget},
};

use crate::{
    app::{
        events::custom::FlowrsEvent,
        model::{popup::popup_area, Model},
        worker::WorkerMessage,
    },
    ui::theme::{
        BORDER_DEFAULT, BORDER_SELECTED, BUTTON_DEFAULT, BUTTON_SELECTED, DEFAULT_STYLE,
        SURFACE_STYLE, TITLE_STYLE,
    },
};

pub struct TriggerDagRunPopUp {
    pub dag_id: String,
    pub confirm: bool,
}

impl TriggerDagRunPopUp {
    pub fn new(dag_id: String) -> Self {
        TriggerDagRunPopUp {
            dag_id,
            confirm: false,
        }
    }
}

impl Model for TriggerDagRunPopUp {
    fn update(&mut self, event: &FlowrsEvent) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        if let FlowrsEvent::Key(key_event) = event {
            match key_event.code {
                KeyCode::Enter => {
                    // On Enter, we always return the key event, so the parent can close the popup
                    // If the confirm flag is set, we also return a WorkerMessage to clear the dag run
                    if self.confirm {
                        return (
                            Some(FlowrsEvent::Key(*key_event)),
                            vec![WorkerMessage::TriggerDagRun {
                                dag_id: self.dag_id.clone(),
                            }],
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

impl Widget for &mut TriggerDagRunPopUp {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        // Smaller popup: 35% width, auto height
        let area = popup_area(area, 40, 30);

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
            .title(" Trigger DAG Run ")
            .border_style(DEFAULT_STYLE)
            .style(SURFACE_STYLE)
            .title_style(TITLE_STYLE);

        let text = Paragraph::new("Trigger a new DAG Run?")
            .style(DEFAULT_STYLE)
            .centered();

        let [_, yes, _, no, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(8),
            Constraint::Length(2),
            Constraint::Length(8),
            Constraint::Fill(1),
        ])
        .areas(options);

        // Yes button
        let (yes_style, yes_border) = if self.confirm {
            (BUTTON_SELECTED, BORDER_SELECTED)
        } else {
            (BUTTON_DEFAULT, BORDER_DEFAULT)
        };
        let yes_btn = Paragraph::new("Yes")
            .style(yes_style)
            .centered()
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .border_style(yes_style.fg(yes_border)),
            );

        // No button
        let (no_style, no_border) = if !self.confirm {
            (BUTTON_SELECTED, BORDER_SELECTED)
        } else {
            (BUTTON_DEFAULT, BORDER_DEFAULT)
        };
        let no_btn = Paragraph::new("No")
            .style(no_style)
            .centered()
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .border_style(no_style.fg(no_border)),
            );

        Clear.render(area, buffer);
        popup_block.render(area, buffer);
        text.render(header, buffer);
        yes_btn.render(yes, buffer);
        no_btn.render(no, buffer);
    }
}
