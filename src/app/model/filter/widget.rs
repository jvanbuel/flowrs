use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

use super::{FilterState, FilterStateMachine};
use crate::ui::theme::theme;

impl FilterStateMachine {
    /// Render the filter widget and update cursor position
    pub fn render_widget(&mut self, area: Rect, buf: &mut Buffer) {
        if !self.is_active() {
            return;
        }

        let (content, cursor_offset) = self.build_display_content();
        let default_style = theme().default_style;

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .title(self.title())
                    .style(default_style),
            )
            .style(default_style);

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
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "value is bounded by terminal/layout dimensions and stays well within the target integer range"
                )]
                for cond in conditions {
                    let cond_text = format!("{}: {} ", cond.field, cond.value);
                    cursor_offset += cond_text.len() as u16;
                    spans.push(Span::styled(
                        cond_text,
                        Style::default().fg(theme().text_muted),
                    ));
                }

                // Render primary field name with colon (like ValueInput)
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "value is bounded by terminal/layout dimensions and stays well within the target integer range"
                )]
                if let Some(field) = self.primary_field() {
                    let field_prefix = format!("{field}: ");
                    cursor_offset += field_prefix.len() as u16;
                    spans.push(Span::styled(
                        field_prefix,
                        Style::default().fg(theme().accent),
                    ));
                }

                // Render typed text
                let typed = &autocomplete.typed;
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "value is bounded by terminal/layout dimensions and stays well within the target integer range"
                )]
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
                        Style::default().fg(theme().text_ghost),
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
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "value is bounded by terminal/layout dimensions and stays well within the target integer range"
                )]
                for cond in conditions {
                    let cond_text = format!("{}: {} ", cond.field, cond.value);
                    cursor_offset += cond_text.len() as u16;
                    spans.push(Span::styled(
                        cond_text,
                        Style::default().fg(theme().text_muted),
                    ));
                }

                // Render colon prefix
                spans.push(Span::styled(":", Style::default().fg(theme().accent)));
                cursor_offset += 1;

                // Render typed attribute name
                let typed = &autocomplete.typed;
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "value is bounded by terminal/layout dimensions and stays well within the target integer range"
                )]
                {
                    cursor_offset += typed.len() as u16;
                }
                spans.push(Span::styled(
                    typed.clone(),
                    Style::default()
                        .fg(theme().accent)
                        .add_modifier(Modifier::BOLD),
                ));

                // Render ghost text for attribute
                if let Some(ghost) = autocomplete.ghost_suffix() {
                    spans.push(Span::styled(
                        ghost.to_string(),
                        Style::default().fg(theme().text_ghost),
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
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "value is bounded by terminal/layout dimensions and stays well within the target integer range"
                )]
                for cond in conditions {
                    let cond_text = format!("{}: {} ", cond.field, cond.value);
                    cursor_offset += cond_text.len() as u16;
                    spans.push(Span::styled(
                        cond_text,
                        Style::default().fg(theme().text_muted),
                    ));
                }

                // Render field name with colon
                let field_prefix = format!("{field}: ");
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "value is bounded by terminal/layout dimensions and stays well within the target integer range"
                )]
                {
                    cursor_offset += field_prefix.len() as u16;
                }
                spans.push(Span::styled(
                    field_prefix,
                    Style::default().fg(theme().accent),
                ));

                // Render typed value
                let typed = &autocomplete.typed;
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "value is bounded by terminal/layout dimensions and stays well within the target integer range"
                )]
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
                        Style::default().fg(theme().text_ghost),
                    ));
                }

                (Line::from(spans), cursor_offset)
            }
        }
    }
}
