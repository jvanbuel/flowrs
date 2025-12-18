pub mod commands_help;
pub mod config;
pub mod dagruns;
pub mod dags;
pub mod error;
pub mod taskinstances;
pub mod warning;

use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::Style,
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::ui::theme::{BORDER_DEFAULT, BORDER_SELECTED, BUTTON_DEFAULT, BUTTON_SELECTED};

/// helper function to create a centered rect using up certain percentage of the available rect `r`
#[allow(dead_code)]
pub fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

/// Create a themed button with consistent styling across popups.
pub fn themed_button(text: &str, selected: bool) -> Paragraph<'_> {
    let (style, border_color) = if selected {
        (BUTTON_SELECTED, BORDER_SELECTED)
    } else {
        (BUTTON_DEFAULT, BORDER_DEFAULT)
    };

    Paragraph::new(text).style(style).centered().block(
        Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .style(style),
    )
}
