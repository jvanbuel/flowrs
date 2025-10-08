use anyhow::Error;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget, Wrap},
};

use super::popup_area;

pub struct ErrorPopup {
    pub errors: Vec<String>,
}

impl ErrorPopup {
    pub fn new(errors: &[Error]) -> Self {
        Self {
            errors: errors.iter().map(|e| e.to_string()).collect(),
        }
    }

    pub fn from_strings(errors: Vec<String>) -> Self {
        Self { errors }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl Widget for &ErrorPopup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.errors.is_empty() {
            return;
        }

        let popup_area = popup_area(area, 80, 50);
        let popup = Block::default()
            .border_type(BorderType::Rounded)
            .title("Errors - Press <Esc> or <q> to close")
            .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red));

        Clear.render(popup_area, buf);

        let mut text = Text::default();
        for (idx, error) in self.errors.iter().enumerate() {
            text.push_line(Line::from(vec![
                Span::styled(
                    format!("Error {}: ", idx + 1),
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(error.as_str(), Style::default().fg(Color::White)),
            ]));
            if idx < self.errors.len() - 1 {
                text.push_line(Line::from(""));
            }
        }

        let error_paragraph = Paragraph::new(text).wrap(Wrap { trim: true }).block(popup);
        error_paragraph.render(popup_area, buf);
    }
}
