pub mod commands_help;
pub mod config;
pub mod dagruns;
pub mod dags;
pub mod taskinstances;

use ratatui::layout::{Constraint, Flex, Layout, Rect};

/// helper function to create a centered rect using up certain percentage of the available rect `r`
#[allow(dead_code)]
pub fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
