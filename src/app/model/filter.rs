use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::Styled,
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

use crate::ui::constants::DEFAULT_STYLE;

#[derive(Clone)]
pub struct CursorState {
    pub position: Position,
}

pub struct Filter {
    pub enabled: bool,
    pub prefix: Option<String>,
    pub cursor: CursorState,
}

impl Filter {
    pub fn new() -> Filter {
        Filter {
            enabled: false,
            prefix: None,
            cursor: CursorState {
                position: Position::default(),
            },
        }
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn prefix(&self) -> &Option<String> {
        &self.prefix
    }

    pub fn reset(&mut self) {
        self.enabled = false;
        self.prefix = None;
    }

    pub fn update(&mut self, key_event: &KeyEvent) {
        match key_event.code {
            KeyCode::Esc | KeyCode::Enter => {
                self.toggle();
            }
            KeyCode::Backspace => {
                if let Some(ref mut prefix) = self.prefix {
                    prefix.pop();
                }
            }
            KeyCode::Char(c) => match self.prefix {
                Some(ref mut prefix) => {
                    prefix.push(c);
                }
                None => {
                    self.prefix = Some(c.to_string());
                }
            },
            _ => {}
        }
    }
    pub fn cursor_position(&self) -> &Position {
        &self.cursor.position
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for &mut Filter {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let filter = self.prefix().clone();
        let filter_text = filter.unwrap_or_default();

        self.cursor.position = Position {
            x: area.x + 1 + filter_text.len() as u16,
            y: area.y + 1,
        };

        let paragraph = Paragraph::new(filter_text)
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .title("filter"),
            )
            .set_style(DEFAULT_STYLE);

        Widget::render(paragraph, area, buf);
    }
}
