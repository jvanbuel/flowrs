use super::Theme;
use super::ThemePalette;

impl Theme {
    /// Catppuccin Latte — the lightest flavor.
    pub fn catppuccin_latte() -> Self {
        let c = &catppuccin::PALETTE.latte.colors;
        Self::build(&ThemePalette {
            text_primary: c.text.into(),
            purple: c.mauve.into(),
            purple_dim: c.overlay1.into(),
            accent: c.teal.into(),
            surface: c.surface0.into(),
            header_bg: c.mauve.into(),
            header_fg: c.base.into(),
            table_header_fg: c.subtext1.into(),
            table_header_bg: c.surface1.into(),
            selected_bg: c.surface2.into(),
            marked_bg: c.crust.into(),
            alt_row_bg: c.mantle.into(),
            schedule_fg: c.yellow.into(),
            dag_active: c.blue.into(),
            border_selected: c.lavender.into(),
            state_success: c.green.into(),
            state_failed: c.red.into(),
            state_running: c.teal.into(),
            state_queued: c.overlay1.into(),
            state_up_for_retry: c.yellow.into(),
            state_up_for_reschedule: c.sky.into(),
            state_skipped: c.pink.into(),
            state_upstream_failed: c.maroon.into(),
        })
    }

    /// Catppuccin Frappé — subdued colors with a muted aesthetic.
    pub fn catppuccin_frappe() -> Self {
        let c = &catppuccin::PALETTE.frappe.colors;
        Self::build(&ThemePalette {
            text_primary: c.text.into(),
            purple: c.mauve.into(),
            purple_dim: c.overlay2.into(),
            accent: c.teal.into(),
            surface: c.surface0.into(),
            header_bg: c.mauve.into(),
            header_fg: c.base.into(),
            table_header_fg: c.subtext1.into(),
            table_header_bg: c.surface1.into(),
            selected_bg: c.surface2.into(),
            marked_bg: c.overlay0.into(),
            alt_row_bg: c.mantle.into(),
            schedule_fg: c.yellow.into(),
            dag_active: c.blue.into(),
            border_selected: c.lavender.into(),
            state_success: c.green.into(),
            state_failed: c.red.into(),
            state_running: c.teal.into(),
            state_queued: c.overlay1.into(),
            state_up_for_retry: c.yellow.into(),
            state_up_for_reschedule: c.sky.into(),
            state_skipped: c.pink.into(),
            state_upstream_failed: c.maroon.into(),
        })
    }

    /// Catppuccin Macchiato — gentle colors for a soothing atmosphere.
    pub fn catppuccin_macchiato() -> Self {
        let c = &catppuccin::PALETTE.macchiato.colors;
        Self::build(&ThemePalette {
            text_primary: c.text.into(),
            purple: c.mauve.into(),
            purple_dim: c.overlay2.into(),
            accent: c.teal.into(),
            surface: c.surface0.into(),
            header_bg: c.mauve.into(),
            header_fg: c.base.into(),
            table_header_fg: c.subtext1.into(),
            table_header_bg: c.surface1.into(),
            selected_bg: c.surface2.into(),
            marked_bg: c.overlay0.into(),
            alt_row_bg: c.mantle.into(),
            schedule_fg: c.yellow.into(),
            dag_active: c.blue.into(),
            border_selected: c.lavender.into(),
            state_success: c.green.into(),
            state_failed: c.red.into(),
            state_running: c.teal.into(),
            state_queued: c.overlay1.into(),
            state_up_for_retry: c.yellow.into(),
            state_up_for_reschedule: c.sky.into(),
            state_skipped: c.pink.into(),
            state_upstream_failed: c.maroon.into(),
        })
    }

    /// Catppuccin Mocha — the darkest flavor with cozy, color-rich accents.
    pub fn catppuccin_mocha() -> Self {
        let c = &catppuccin::PALETTE.mocha.colors;
        Self::build(&ThemePalette {
            text_primary: c.text.into(),
            purple: c.mauve.into(),
            purple_dim: c.overlay2.into(),
            accent: c.teal.into(),
            surface: c.surface0.into(),
            header_bg: c.mauve.into(),
            header_fg: c.base.into(),
            table_header_fg: c.subtext1.into(),
            table_header_bg: c.surface1.into(),
            selected_bg: c.surface2.into(),
            marked_bg: c.overlay0.into(),
            alt_row_bg: c.mantle.into(),
            schedule_fg: c.yellow.into(),
            dag_active: c.blue.into(),
            border_selected: c.lavender.into(),
            state_success: c.green.into(),
            state_failed: c.red.into(),
            state_running: c.teal.into(),
            state_queued: c.overlay1.into(),
            state_up_for_retry: c.yellow.into(),
            state_up_for_reschedule: c.sky.into(),
            state_skipped: c.pink.into(),
            state_upstream_failed: c.maroon.into(),
        })
    }
}
