use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Text,
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget, Wrap},
};

use super::popup_area;

pub struct Command<'a> {
    pub name: &'a str,
    pub key_binding: &'a str,
    pub description: &'a str,
}
pub struct CommandPopUp<'a> {
    pub title: String,
    pub commands: Vec<Command<'a>>,
}

impl Widget for &CommandPopUp<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup_area = popup_area(area, 80, 80);
        let popup = Block::default()
            .border_type(BorderType::Rounded)
            .title(self.title.as_str())
            .borders(Borders::ALL);

        Clear.render(popup_area, buf);

        let text = Text::from_iter(
            self.commands
                .iter()
                .map(|c| format!("<{}>: {} - {}", c.key_binding, c.name, c.description)),
        );

        let command_paragraph = Paragraph::new(text).wrap(Wrap { trim: true }).block(popup);
        command_paragraph.render(popup_area, buf);
    }
}
