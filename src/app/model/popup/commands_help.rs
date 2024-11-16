use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Text,
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};

use super::popup_area;

pub struct Command<'a> {
    pub name: &'a str,
    pub key_binding: &'a str,
    pub description: &'a str,
}
pub struct CommandPopUp<'a, const N: usize> {
    pub title: &'a str,
    pub commands: [Command<'a>; N],
}

impl<const N: usize> Widget for &CommandPopUp<'_, N> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup_area = popup_area(area, 80, 80);
        let popup = Block::default().title(self.title).borders(Borders::ALL);

        Clear.render(popup_area, buf);

        let text = Text::from_iter(
            self.commands
                .iter()
                .map(|c| format!("{}: {} - {}", c.key_binding, c.name, c.description)),
        );

        let command_paragraph = Paragraph::new(text).wrap(Wrap { trim: true }).block(popup);
        command_paragraph.render(popup_area, buf);
    }
}
