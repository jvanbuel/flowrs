use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Clear, Paragraph, Widget, Wrap},
};

use super::popup_area;
use crate::ui::common::titled_popup_block;
use crate::ui::theme::theme;

#[derive(Debug)]
pub struct WarningPopup {
    pub warnings: Vec<String>,
}

impl WarningPopup {
    pub const fn new(warnings: Vec<String>) -> Self {
        Self { warnings }
    }
}

impl Widget for &WarningPopup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.warnings.is_empty() {
            return;
        }

        let t = theme();
        let warning_color = t.state_up_for_retry;

        let popup_area = popup_area(area, 80, 50);
        let popup = titled_popup_block("Warning - Press <Esc> or <q> to close", warning_color);

        Clear.render(popup_area, buf);

        let mut text = Text::default();
        for (idx, warning) in self.warnings.iter().enumerate() {
            // Split the warning by newlines to properly render multi-line warnings
            for (line_idx, line) in warning.lines().enumerate() {
                if line_idx == 0 {
                    // First line includes the "Warning N: " prefix
                    text.push_line(Line::from(vec![
                        Span::styled(
                            format!("Warning {}: ", idx + 1),
                            Style::default()
                                .fg(warning_color)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(line, Style::default().fg(t.text_primary)),
                    ]));
                } else {
                    text.push_line(Line::from(Span::styled(
                        line,
                        Style::default().fg(t.text_primary),
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
