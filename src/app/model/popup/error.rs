use anyhow::Error;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget, Wrap},
};

use super::popup_area;
use crate::ui::theme::theme;

pub struct ErrorPopup {
    pub errors: Vec<String>,
}

impl ErrorPopup {
    pub fn new(errors: &[Error]) -> Self {
        Self {
            errors: errors
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
        }
    }

    pub const fn from_strings(errors: Vec<String>) -> Self {
        Self { errors }
    }
}

impl Widget for &ErrorPopup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.errors.is_empty() {
            return;
        }

        let t = theme();
        let error_color = t.state_failed;

        let popup_area = popup_area(area, 80, 50);
        let popup = Block::default()
            .border_type(BorderType::Rounded)
            .title("Errors - Press <Esc> or <q> to close")
            .title_style(Style::default().fg(error_color).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(t.border_style)
            .style(t.default_style);

        Clear.render(popup_area, buf);

        let mut text = Text::default();
        for (idx, error) in self.errors.iter().enumerate() {
            let mut lines = error.split('\n');
            if let Some(first_line) = lines.next() {
                text.push_line(Line::from(vec![
                    Span::styled(
                        format!("Error {}: ", idx + 1),
                        Style::default()
                            .fg(error_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        first_line.to_string(),
                        Style::default().fg(t.text_primary),
                    ),
                ]));
            }
            for line in lines {
                text.push_line(Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(t.text_primary),
                )));
            }
            if idx < self.errors.len() - 1 {
                text.push_line(Line::from(""));
            }
        }

        let error_paragraph = Paragraph::new(text).wrap(Wrap { trim: true }).block(popup);
        error_paragraph.render(popup_area, buf);
    }
}
