//! Centralized theme module for Flowrs TUI.
//!
//! Call [`init_theme`] once at startup, then access the active theme anywhere
//! via [`theme()`]:
//! ```ignore
//! let t = theme();
//! let style = t.border_style;
//! ```

mod presets;

use std::sync::OnceLock;

use flowrs_config::Theme as ThemePreset;
use ratatui::style::{Color, Style};

/// Complete theme definition with all colors and pre-built styles.
#[derive(Debug)]
#[allow(dead_code)]
pub struct Theme {
    pub text_primary: Color,
    pub purple: Color,
    pub purple_dim: Color,
    pub accent: Color,
    pub surface: Color,
    pub header_bg: Color,
    pub header_fg: Color,
    pub table_header_fg: Color,
    pub table_header_bg: Color,
    pub selected_bg: Color,
    pub marked_bg: Color,
    pub alt_row_bg: Color,
    pub dag_active: Color,
    pub schedule_fg: Color,
    pub state_success: Color,
    pub state_failed: Color,
    pub state_running: Color,
    pub state_queued: Color,
    pub state_up_for_retry: Color,
    pub state_up_for_reschedule: Color,
    pub state_skipped: Color,
    pub state_upstream_failed: Color,
    pub default_style: Style,
    pub border_style: Style,
    pub title_style: Style,
    pub table_header_style: Style,
    pub selected_row_style: Style,
    pub marked_style: Style,
    pub alt_row_style: Style,
    pub surface_style: Style,
    pub button_default: Style,
    pub button_selected: Style,
    pub border_default: Color,
    pub border_selected: Color,
}

static THEME: OnceLock<Theme> = OnceLock::new();

/// Initialize the global theme. Must be called at startup before rendering.
///
/// Idempotent: subsequent calls are silently ignored if the theme is already set.
pub fn init_theme(preset: ThemePreset) {
    let theme = match preset {
        ThemePreset::Auto => {
            if detect_light_terminal() {
                log::info!("Auto-detected light terminal, using light theme");
                Theme::light()
            } else {
                log::info!("Auto-detected dark terminal, using dark theme");
                Theme::dark()
            }
        }
        ThemePreset::Dark => {
            log::info!("Using dark theme (configured)");
            Theme::dark()
        }
        ThemePreset::Light => {
            log::info!("Using light theme (configured)");
            Theme::light()
        }
        ThemePreset::CatppuccinLatte => {
            log::info!("Using Catppuccin Latte theme (configured)");
            Theme::catppuccin_latte()
        }
        ThemePreset::CatppuccinFrappe => {
            log::info!("Using Catppuccin Frappé theme (configured)");
            Theme::catppuccin_frappe()
        }
        ThemePreset::CatppuccinMacchiato => {
            log::info!("Using Catppuccin Macchiato theme (configured)");
            Theme::catppuccin_macchiato()
        }
        ThemePreset::CatppuccinMocha => {
            log::info!("Using Catppuccin Mocha theme (configured)");
            Theme::catppuccin_mocha()
        }
    };
    THEME.set(theme).ok();
}

/// Returns the active theme. Panics if [`init_theme`] has not been called.
pub fn theme() -> &'static Theme {
    #[cfg(test)]
    return THEME.get_or_init(Theme::dark);
    #[cfg(not(test))]
    THEME
        .get()
        .expect("init_theme must be called before theme()")
}

fn detect_light_terminal() -> bool {
    use terminal_colorsaurus::{theme_mode, QueryOptions};

    match theme_mode(QueryOptions::default()) {
        Ok(mode) => mode == terminal_colorsaurus::ThemeMode::Light,
        Err(e) => {
            log::debug!("Could not detect terminal color scheme: {e}, defaulting to dark");
            false
        }
    }
}
