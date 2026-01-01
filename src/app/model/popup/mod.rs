pub mod commands_help;
pub mod config;
pub mod dagruns;
pub mod dags;
pub mod error;
pub mod taskinstances;
pub mod warning;

use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::Style,
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

use crate::ui::theme::{BORDER_DEFAULT, BORDER_SELECTED, BUTTON_DEFAULT, BUTTON_SELECTED};

use self::{commands_help::CommandPopUp, error::ErrorPopup};
use super::KeyResult;

/// A unified popup type that can hold any of the common popup types.
///
/// This simplifies popup handling by providing a single field that can
/// represent error popups, command help popups, or panel-specific popups.
#[derive(Default)]
pub enum Popup<T = ()> {
    /// No popup is shown
    #[default]
    None,
    /// Error popup (common to all panels)
    Error(ErrorPopup),
    /// Commands help popup (common to all panels)
    Commands(&'static CommandPopUp<'static>),
    /// Panel-specific popup
    Custom(T),
}

impl<T> Popup<T> {
    /// Returns true if no popup is shown
    pub fn is_none(&self) -> bool {
        matches!(self, Popup::None)
    }

    /// Handle common popup dismissal keys (error and commands popups)
    pub fn handle_dismiss(&mut self, key_code: KeyCode) -> KeyResult {
        match self {
            Popup::Error(_) => {
                if matches!(key_code, KeyCode::Char('q') | KeyCode::Esc) {
                    *self = Popup::None;
                }
                KeyResult::Consumed
            }
            Popup::Commands(_) => {
                if matches!(
                    key_code,
                    KeyCode::Char('q' | '?') | KeyCode::Esc | KeyCode::Enter
                ) {
                    *self = Popup::None;
                }
                KeyResult::Consumed
            }
            // None and Custom popups are not handled here (Custom needs custom handling)
            Popup::None | Popup::Custom(_) => KeyResult::Ignored,
        }
    }

    /// Show error popup
    pub fn show_error(&mut self, errors: Vec<String>) {
        *self = Popup::Error(ErrorPopup::from_strings(errors));
    }

    /// Show commands popup
    pub fn show_commands(&mut self, commands: &'static CommandPopUp<'static>) {
        *self = Popup::Commands(commands);
    }

    /// Show custom popup
    pub fn show_custom(&mut self, popup: T) {
        *self = Popup::Custom(popup);
    }

    /// Get mutable reference to custom popup if present
    pub fn custom_mut(&mut self) -> Option<&mut T> {
        match self {
            Popup::Custom(p) => Some(p),
            _ => None,
        }
    }

    /// Close the popup
    pub fn close(&mut self) {
        *self = Popup::None;
    }
}

impl<T> Widget for &Popup<T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            Popup::Error(e) => e.render(area, buf),
            Popup::Commands(c) => c.render(area, buf),
            // None and Custom popups don't render here (Custom is rendered separately by the model)
            Popup::None | Popup::Custom(_) => {}
        }
    }
}

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
