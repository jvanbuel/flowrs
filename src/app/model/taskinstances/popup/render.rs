use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget},
};

use crate::{
    app::model::popup::{popup_area, themed_button},
    ui::theme::theme,
};

use super::clear::ClearTaskInstancePopup;
use super::mark::{MarkState, MarkTaskInstancePopup};

impl Widget for &mut ClearTaskInstancePopup {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let t = theme();
        // Smaller popup: 40% width, auto height
        let area = popup_area(area, 40, 30);

        let popup_block = Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(t.border_style)
            .style(t.surface_style);

        // Use inner area for content layout to avoid overlapping the border
        let inner = popup_block.inner(area);

        let [_, header, options, _] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .flex(Flex::Center)
        .areas(inner);

        let message = if self.task_ids.len() == 1 {
            "Clear this Task Instance?".to_string()
        } else {
            format!("Clear {} Task Instances?", self.task_ids.len())
        };
        let text = Paragraph::new(message).style(t.default_style).centered();

        let [_, yes, _, no, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(8),
            Constraint::Length(2),
            Constraint::Length(8),
            Constraint::Fill(1),
        ])
        .areas(options);

        let yes_btn = themed_button("Yes", self.selected_button.is_yes());
        let no_btn = themed_button("No", !self.selected_button.is_yes());

        Clear.render(area, buffer);
        popup_block.render(area, buffer);
        text.render(header, buffer);
        yes_btn.render(yes, buffer);
        no_btn.render(no, buffer);
    }
}

impl Widget for &mut MarkTaskInstancePopup {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let t = theme();
        // Smaller popup: 35% width, auto height
        let area = popup_area(area, 35, 30);

        let [_, header, options, _] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .flex(Flex::Center)
        .areas(area);

        let popup_block = Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(t.border_style)
            .style(t.surface_style);

        let text = Paragraph::new("Mark status as")
            .style(t.default_style)
            .centered();

        let [_, success, _, failed, _, skipped, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(11),
            Constraint::Length(2),
            Constraint::Length(10),
            Constraint::Length(2),
            Constraint::Length(11),
            Constraint::Fill(1),
        ])
        .areas(options);

        // Success button
        let (success_style, success_border) = if self.status == MarkState::Success {
            (t.button_selected, t.border_selected)
        } else {
            (t.button_default, t.border_default)
        };
        let success_btn = Paragraph::new("Success")
            .style(success_style)
            .centered()
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .border_style(success_style.fg(success_border)),
            );

        // Failed button
        let (failed_style, failed_border) = if self.status == MarkState::Failed {
            (t.button_selected, t.border_selected)
        } else {
            (t.button_default, t.border_default)
        };
        let failed_btn = Paragraph::new("Failed")
            .style(failed_style)
            .centered()
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .border_style(failed_style.fg(failed_border)),
            );

        // Skipped button
        let (skipped_style, skipped_border) = if self.status == MarkState::Skipped {
            (t.button_selected, t.border_selected)
        } else {
            (t.button_default, t.border_default)
        };
        let skipped_btn = Paragraph::new("Skipped")
            .style(skipped_style)
            .centered()
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .border_style(skipped_style.fg(skipped_border)),
            );

        Clear.render(area, buffer);
        popup_block.render(area, buffer);
        text.render(header, buffer);
        success_btn.render(success, buffer);
        failed_btn.render(failed, buffer);
        skipped_btn.render(skipped, buffer);
    }
}
