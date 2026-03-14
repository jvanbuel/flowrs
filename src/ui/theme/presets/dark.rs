use ratatui::style::Color;

use super::Theme;
use super::ThemePalette;

impl Theme {
    /// Dark theme preset — the original flowrs color scheme.
    pub fn dark() -> Self {
        Self::build(&ThemePalette {
            text_primary: Color::Rgb(220, 220, 230),    // #DCDCE6
            purple: Color::Rgb(138, 118, 255),          // #8A76FF
            purple_dim: Color::Rgb(90, 80, 160),        // #5A50A0
            accent: Color::Rgb(0, 220, 160),            // #00DCA0
            surface: Color::Rgb(50, 55, 70),            // #323746
            header_bg: Color::Rgb(138, 118, 255),       // #8A76FF
            header_fg: Color::Rgb(20, 20, 30),          // #14141E
            table_header_fg: Color::Rgb(200, 200, 210), // #C8C8D2
            table_header_bg: Color::Rgb(45, 50, 65),    // #2D3241
            selected_bg: Color::Rgb(0, 80, 60),         // #00503C
            marked_bg: Color::Rgb(80, 50, 120),         // #503278
            alt_row_bg: Color::Rgb(30, 32, 42),         // #1E202A
            schedule_fg: Color::LightYellow,
            dag_active: Color::Rgb(30, 144, 255),     // #1E90FF
            border_selected: Color::Rgb(0, 220, 160), // accent
            state_success: Color::Rgb(0, 153, 0),     // #009900
            state_failed: Color::Rgb(255, 107, 107),  // #FF6B6B
            state_running: Color::Rgb(155, 255, 155), // #9BFF9B
            state_queued: Color::Rgb(128, 128, 128),  // #808080
            state_up_for_retry: Color::Rgb(255, 179, 71), // #FFB347
            state_up_for_reschedule: Color::Rgb(111, 231, 219), // #6FE7DB
            state_skipped: Color::Rgb(255, 142, 198), // #FF8EC6
            state_upstream_failed: Color::Rgb(255, 165, 0), // #FFA500
        })
    }
}
