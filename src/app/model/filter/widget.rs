use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

use super::{FilterState, FilterStateMachine};
use crate::ui::constants::DEFAULT_STYLE;
use crate::ui::theme::ACCENT;

impl FilterStateMachine {
    /// Render the filter widget and update cursor position
    pub fn render_widget(&mut self, area: Rect, buf: &mut Buffer) {
        if !self.is_active() {
            return;
        }

        let (content, cursor_offset) = self.build_display_content();

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .title(self.title())
                    .style(DEFAULT_STYLE),
            )
            .style(DEFAULT_STYLE);

        Widget::render(paragraph, area, buf);

        // Update cursor position
        self.cursor_position = Position {
            x: area.x + 1 + cursor_offset,
            y: area.y + 1,
        };
    }

    const fn title(&self) -> &'static str {
        match &self.state {
            FilterState::AttributeSelection { .. } => "filter (select attribute)",
            FilterState::Inactive
            | FilterState::Default { .. }
            | FilterState::ValueInput { .. } => "filter",
        }
    }

    fn build_display_content(&self) -> (Line<'static>, u16) {
        match &self.state {
            FilterState::Inactive => (Line::from(""), 0),

            FilterState::Default {
                autocomplete,
                conditions,
            } => {
                let mut spans = Vec::new();
                let mut cursor_offset: u16 = 0;

                // Render confirmed conditions
                #[allow(clippy::cast_possible_truncation)]
                for cond in conditions {
                    let cond_text = format!("{}: {} ", cond.field, cond.value);
                    cursor_offset += cond_text.len() as u16;
                    spans.push(Span::styled(cond_text, Style::default().fg(Color::Gray)));
                }

                // Render primary field name with colon (like ValueInput)
                #[allow(clippy::cast_possible_truncation)]
                if let Some(field) = self.primary_field() {
                    let field_prefix = format!("{field}: ");
                    cursor_offset += field_prefix.len() as u16;
                    spans.push(Span::styled(field_prefix, Style::default().fg(ACCENT)));
                }

                // Render typed text
                let typed = &autocomplete.typed;
                #[allow(clippy::cast_possible_truncation)]
                {
                    cursor_offset += typed.len() as u16;
                }
                spans.push(Span::styled(
                    typed.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                ));

                // Render ghost text
                if let Some(ghost) = autocomplete.ghost_suffix() {
                    spans.push(Span::styled(
                        ghost.to_string(),
                        Style::default().fg(Color::DarkGray),
                    ));
                }

                (Line::from(spans), cursor_offset)
            }

            FilterState::AttributeSelection {
                autocomplete,
                conditions,
            } => {
                let mut spans = Vec::new();
                let mut cursor_offset: u16 = 0;

                // Render confirmed conditions
                #[allow(clippy::cast_possible_truncation)]
                for cond in conditions {
                    let cond_text = format!("{}: {} ", cond.field, cond.value);
                    cursor_offset += cond_text.len() as u16;
                    spans.push(Span::styled(cond_text, Style::default().fg(Color::Gray)));
                }

                // Render colon prefix
                spans.push(Span::styled(":", Style::default().fg(ACCENT)));
                cursor_offset += 1;

                // Render typed attribute name
                let typed = &autocomplete.typed;
                #[allow(clippy::cast_possible_truncation)]
                {
                    cursor_offset += typed.len() as u16;
                }
                spans.push(Span::styled(
                    typed.clone(),
                    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                ));

                // Render ghost text for attribute
                if let Some(ghost) = autocomplete.ghost_suffix() {
                    spans.push(Span::styled(
                        ghost.to_string(),
                        Style::default().fg(Color::DarkGray),
                    ));
                }

                (Line::from(spans), cursor_offset)
            }

            FilterState::ValueInput {
                field,
                autocomplete,
                conditions,
                ..
            } => {
                let mut spans = Vec::new();
                let mut cursor_offset: u16 = 0;

                // Render confirmed conditions
                #[allow(clippy::cast_possible_truncation)]
                for cond in conditions {
                    let cond_text = format!("{}: {} ", cond.field, cond.value);
                    cursor_offset += cond_text.len() as u16;
                    spans.push(Span::styled(cond_text, Style::default().fg(Color::Gray)));
                }

                // Render field name with colon
                let field_prefix = format!("{field}: ");
                #[allow(clippy::cast_possible_truncation)]
                {
                    cursor_offset += field_prefix.len() as u16;
                }
                spans.push(Span::styled(field_prefix, Style::default().fg(ACCENT)));

                // Render typed value
                let typed = &autocomplete.typed;
                #[allow(clippy::cast_possible_truncation)]
                {
                    cursor_offset += typed.len() as u16;
                }
                spans.push(Span::styled(
                    typed.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                ));

                // Render ghost text for value
                if let Some(ghost) = autocomplete.ghost_suffix() {
                    spans.push(Span::styled(
                        ghost.to_string(),
                        Style::default().fg(Color::DarkGray),
                    ));
                }

                (Line::from(spans), cursor_offset)
            }
        }
    }
}
