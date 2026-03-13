use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Text},
    widgets::{
        Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, StatefulWidget,
        Tabs, Widget, Wrap,
    },
};

use crate::ui::theme::{ACCENT, BORDER_STYLE, DEFAULT_STYLE, TITLE_STYLE};

use super::LogModel;

impl Widget for &mut LogModel {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        if self.all.is_empty() {
            Paragraph::new("No logs available")
                .style(DEFAULT_STYLE)
                .block(
                    Block::default()
                        .border_type(BorderType::Rounded)
                        .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                        .border_style(BORDER_STYLE),
                )
                .render(area, buffer);
            return;
        }

        let tab_titles = (0..self.all.len())
            .map(|i| format!("Task {}", i + 1))
            .collect::<Vec<String>>();

        let tabs = Tabs::new(tab_titles)
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                    .border_style(BORDER_STYLE),
            )
            .select(self.current % self.all.len())
            .highlight_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
            .style(DEFAULT_STYLE);

        // Render the tabs
        tabs.render(area, buffer);

        // Define the layout for content under the tabs
        let chunks = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        if let Some(log) = self.all.get(self.current % self.all.len()) {
            let mut content = Text::default();
            for line in log.content.lines() {
                content.push_line(Line::raw(line));
            }

            let line_count = self.current_line_count();
            let scroll_pos = self.scroll_mode.position(line_count);
            self.vertical_scroll_state = self.vertical_scroll_state.position(scroll_pos);

            #[allow(clippy::cast_possible_truncation)]
            let paragraph = Paragraph::new(content)
                .block(
                    Block::default()
                        .border_type(BorderType::Plain)
                        .borders(Borders::ALL)
                        .title(" Content ")
                        .title_bottom(if self.scroll_mode.is_following() {
                            " [F]ollow: ON - auto-scrolling "
                        } else {
                            " [F]ollow: OFF - press G to resume "
                        })
                        .border_style(BORDER_STYLE)
                        .title_style(TITLE_STYLE),
                )
                .wrap(Wrap { trim: true })
                .style(DEFAULT_STYLE)
                .scroll((scroll_pos as u16, 0));

            // Render the selected log's content
            paragraph.render(chunks[1], buffer);

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            scrollbar.render(chunks[1], buffer, &mut self.vertical_scroll_state);
        }

        if let Some(error_popup) = &self.error_popup {
            error_popup.render(area, buffer);
        }
    }
}
