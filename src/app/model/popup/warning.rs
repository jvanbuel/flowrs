use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget, Wrap},
};

use super::popup_area;

pub struct WarningPopup {
    pub warnings: Vec<String>,
}

impl WarningPopup {
    pub fn new(warnings: Vec<String>) -> Self {
        Self { warnings }
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

impl Widget for &WarningPopup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.warnings.is_empty() {
            return;
        }

        let popup_area = popup_area(area, 80, 50);
        let popup = Block::default()
            .border_type(BorderType::Rounded)
            .title("Warning - Press <Esc> or <q> to close")
            .title_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        Clear.render(popup_area, buf);

        let mut text = Text::default();
        for (idx, warning) in self.warnings.iter().enumerate() {
            // Split the warning by newlines to properly render multi-line warnings
            let lines: Vec<&str> = warning.split('\n').collect();
            for (line_idx, line) in lines.iter().enumerate() {
                if line_idx == 0 {
                    // First line includes the "Warning N: " prefix
                    text.push_line(Line::from(vec![
                        Span::styled(
                            format!("Warning {}: ", idx + 1),
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(*line, Style::default().fg(Color::White)),
                    ]));
                } else {
                    // Subsequent lines are just white text
                    text.push_line(Line::from(Span::styled(
                        *line,
                        Style::default().fg(Color::White),
                    )));
                }
            }
            if idx < self.warnings.len() - 1 {
                text.push_line(Line::from(""));
            }
        }

        let warning_paragraph = Paragraph::new(text).wrap(Wrap { trim: true }).block(popup);
        warning_paragraph.render(popup_area, buf);
    }
}
