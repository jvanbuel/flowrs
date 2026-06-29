use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget, Wrap},
};

use crate::app::model::popup::popup_area;
use crate::ui::theme::theme;

use super::CommandPopUp;

impl Widget for &CommandPopUp<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let t = theme();
        let popup_area = popup_area(area, 80, 80);
        let popup = Block::default()
            .border_type(BorderType::Rounded)
            .title(self.title.as_str())
            .title_style(t.title_style)
            .borders(Borders::ALL)
            .border_style(t.border_style)
            .style(t.default_style);

        Clear.render(popup_area, buf);

        let text = self
            .commands
            .iter()
            .map(|c| {
                Line::from(vec![
                    Span::styled(
                        format!("<{}>: ", c.key_binding),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{} - {}", c.name, c.description),
                        Style::default().fg(theme().text_primary),
                    ),
                ])
            })
            .collect::<Text>();

        let command_paragraph = Paragraph::new(text).wrap(Wrap { trim: true }).block(popup);
        command_paragraph.render(popup_area, buf);
    }
}
