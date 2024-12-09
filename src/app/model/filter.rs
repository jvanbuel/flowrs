use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Styled,
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

use crate::ui::constants::DEFAULT_STYLE;

pub struct Filter {
    pub enabled: bool,
    pub prefix: Option<String>,
}

impl Filter {
    pub fn new() -> Filter {
        Filter {
            enabled: false,
            prefix: None,
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
}

impl Default for Filter {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for &Filter {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let filter = self.prefix().clone();

        let paragraph = Paragraph::new(filter.unwrap_or("".to_string()))
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
