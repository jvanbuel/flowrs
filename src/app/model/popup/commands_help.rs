use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style, Stylize},
    text::{Line, Span, Text},
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
                        Style::default().dark_gray(),
                    ),
                ])
            })
            .collect::<Text>();

        let command_paragraph = Paragraph::new(text).wrap(Wrap { trim: true }).block(popup);
        command_paragraph.render(popup_area, buf);
    }
}

pub struct DefaultCommands(pub Vec<Command<'static>>);

impl DefaultCommands {
    pub fn new() -> Self {
        Self(vec![
            Command {
                name: "Enter",
                key_binding: "Enter",
                description: "Open the selected item",
            },
            Command {
                name: "Filter",
                key_binding: "/",
                description: "Filter items",
            },
            Command {
                name: "Open",
                key_binding: "o",
                description: "Open the selected item in the browser",
            },
            Command {
                name: "Previous",
                key_binding: "k / Up",
                description: "Move to the previous item",
            },
            Command {
                name: "Next",
                key_binding: "j / Down",
                description: "Move to the next item",
            },
            Command {
                name: "Previous tab",
                key_binding: "h / Left / Esc",
                description: "Move to the previous tab",
            },
            Command {
                name: "Next tab",
                key_binding: "l / Right",
                description: "Move to the next tab",
            },
            Command {
                name: "Help",
                key_binding: "?",
                description: "Show help",
            },
            Command {
                name: "Quit",
                key_binding: "q / Ctrl-c",
                description: "Quit",
            },
        ])
    }
}
