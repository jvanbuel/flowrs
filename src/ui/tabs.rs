//! Custom tab bar widget with Lip Gloss-style three-sided tabs.
//!
//! The active tab "opens" into the content below by having no bottom border,
//! while inactive tabs have a complete bottom border.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::Widget,
};
use unicode_width::UnicodeWidthStr;

use super::theme::{BORDER_STYLE, PURPLE, TEXT_PRIMARY};

/// Tab definition with label and icon
pub struct Tab {
    pub icon: &'static str,
    pub label: &'static str,
}

impl Tab {
    pub const fn new(icon: &'static str, label: &'static str) -> Self {
        Self { icon, label }
    }

    /// Returns the display width of the tab content (icon + space + label)
    fn content_width(&self) -> usize {
        // icon width + space + label width
        self.icon.width() + 1 + self.label.width()
    }

    /// Returns the total width including borders and padding
    fn total_width(&self) -> usize {
        // │ + space + content + space + │
        1 + 1 + self.content_width() + 1 + 1
    }
}

/// The five panel tabs
pub const TABS: [Tab; 5] = [
    Tab::new("⚙", "Config"),
    Tab::new("𖣘", "DAGs"),
    Tab::new("▶", "Runs"),
    Tab::new("◉", "Tasks"),
    Tab::new("≣", "Logs"),
];

/// Tab bar widget that renders tabs with three-sided borders.
/// The active tab has no bottom border, creating a visual connection to content below.
pub struct TabBar {
    /// Index of the currently active tab
    active: usize,
    /// Style for the active tab
    active_style: Style,
    /// Style for inactive tabs
    inactive_style: Style,
    /// Style for borders
    border_style: Style,
}

impl TabBar {
    pub fn new(active: usize) -> Self {
        Self {
            active,
            active_style: Style::default().fg(TEXT_PRIMARY).bg(PURPLE),
            inactive_style: Style::default().fg(TEXT_PRIMARY),
            border_style: BORDER_STYLE,
        }
    }
}

impl Widget for TabBar {
    #[allow(clippy::cast_possible_truncation)] // Tab widths are small, truncation won't occur
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 3 {
            return; // Need at least 3 rows for tabs + border
        }

        // Track x position as we render each tab
        let mut x = area.x;

        // Line positions
        let y1 = area.y;      // Tab tops
        let y2 = area.y + 1;  // Tab content
        let y3 = area.y + 2;  // Tab bottoms / shared border line

        let tabs_total_width: u16 = TABS.iter().map(Tab::total_width).sum::<usize>() as u16;
        let has_right_extension = tabs_total_width < area.width;

        for (idx, tab) in TABS.iter().enumerate() {
            let tab_width = tab.total_width();
            let is_active = idx == self.active;
            let is_first = idx == 0;
            let style = if is_active {
                self.active_style
            } else {
                self.inactive_style
            };

            // Ensure we don't render outside the area
            if x + tab_width as u16 > area.x + area.width {
                break;
            }

            // Line 1: Top border ╭───╮
            buf.set_string(x, y1, "╭", self.border_style);
            for i in 1..tab_width - 1 {
                buf.set_string(x + i as u16, y1, "─", self.border_style);
            }
            buf.set_string(x + tab_width as u16 - 1, y1, "╮", self.border_style);

            // Line 2: Content │ icon label │
            buf.set_string(x, y2, "│", self.border_style);
            let content = format!(" {} {} ", tab.icon, tab.label);
            buf.set_string(x + 1, y2, &content, style);
            buf.set_string(x + tab_width as u16 - 1, y2, "│", self.border_style);

            // Line 3: Bottom border - different for active vs inactive
            // Left edge character
            let left_char = if is_first && is_active {
                "│" // Active first tab: just vertical, no horizontal (gap starts)
            } else if is_first {
                "├" // Inactive first tab: T-junction connecting to tab bottom
            } else if is_active {
                "┘" // Active non-first tab: terminates line from left
            } else {
                "┴" // Inactive non-first tab: connects to horizontal line
            };
            buf.set_string(x, y3, left_char, self.border_style);

            // Right edge character
            if is_active {
                // Active tab: no bottom, just spaces
                for i in 1..tab_width - 1 {
                    buf.set_string(x + i as u16, y3, " ", Style::default());
                }
                // Right edge: └ connects up (tab border) and right (to next tab or extension)
                buf.set_string(x + tab_width as u16 - 1, y3, "└", self.border_style);
            } else {
                // Inactive tab: complete bottom border
                for i in 1..tab_width - 1 {
                    buf.set_string(x + i as u16, y3, "─", self.border_style);
                }
                // Right edge: T-junction connecting up (tab border), left (tab bottom), right (next tab or extension)
                buf.set_string(x + tab_width as u16 - 1, y3, "┴", self.border_style);
            }

            x += tab_width as u16;
        }

        // Right side: extend border from end of tabs to area.x + area.width
        if has_right_extension {
            for i in tabs_total_width..area.width - 1 {
                buf.set_string(area.x + i, y3, "─", self.border_style);
            }
            buf.set_string(area.x + area.width - 1, y3, "╮", self.border_style);
        }
    }
}

/// Returns the height needed for the tab bar (3 lines: top, content, bottom/shared border)
pub const TAB_BAR_HEIGHT: u16 = 3;
