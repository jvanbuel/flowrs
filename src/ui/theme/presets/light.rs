use ratatui::style::Color;

use super::Theme;
use super::ThemePalette;

impl Theme {
    /// Light theme preset — designed for terminals with light backgrounds.
    pub fn light() -> Self {
        Self::build(&ThemePalette {
            text_primary: Color::Rgb(46, 46, 58),               // #2E2E3A
            purple: Color::Rgb(110, 86, 230),                   // #6E56E6
            purple_dim: Color::Rgb(155, 143, 204),              // #9B8FCC
            accent: Color::Rgb(0, 140, 100),                    // #008C64
            surface: Color::Rgb(240, 240, 245),                 // #F0F0F5
            header_bg: Color::Rgb(110, 86, 230),                // #6E56E6
            header_fg: Color::Rgb(255, 255, 255),               // #FFFFFF
            table_header_fg: Color::Rgb(58, 58, 74),            // #3A3A4A
            table_header_bg: Color::Rgb(232, 232, 240),         // #E8E8F0
            selected_bg: Color::Rgb(179, 240, 224),             // #B3F0E0
            marked_bg: Color::Rgb(216, 200, 255),               // #D8C8FF
            alt_row_bg: Color::Rgb(245, 245, 250),              // #F5F5FA
            schedule_fg: Color::Rgb(160, 120, 0),               // dark gold
            dag_active: Color::Rgb(30, 144, 255),               // #1E90FF
            border_selected: Color::Rgb(0, 140, 100),           // accent
            state_success: Color::Rgb(0, 153, 0),               // #009900
            state_failed: Color::Rgb(255, 107, 107),            // #FF6B6B
            state_running: Color::Rgb(155, 255, 155),           // #9BFF9B
            state_queued: Color::Rgb(128, 128, 128),            // #808080
            state_up_for_retry: Color::Rgb(255, 179, 71),       // #FFB347
            state_up_for_reschedule: Color::Rgb(111, 231, 219), // #6FE7DB
            state_skipped: Color::Rgb(255, 142, 198),           // #FF8EC6
            state_upstream_failed: Color::Rgb(255, 165, 0),     // #FFA500
        })
    }
}
