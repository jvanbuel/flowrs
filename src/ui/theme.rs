//! Centralized theme module for Flowrs TUI.
//!
//! Provides dark and light theme presets with automatic terminal detection.
//! The active theme is stored in a global `OnceLock` and initialized at startup.
//!
//! # Usage
//! Access the active theme via the `theme()` function:
//! ```ignore
//! use crate::ui::theme::theme;
//! let t = theme();
//! let style = t.border_style;
//! ```

use std::sync::OnceLock;

use flowrs_config::ThemeMode;
use ratatui::style::{Color, Modifier, Style};

// =============================================================================
// THEME STRUCT
// =============================================================================

/// Complete theme definition with all colors and pre-built styles.
#[derive(Debug)]
#[allow(dead_code)]
pub struct Theme {
    // -- Colors --
    /// Main text color
    pub text_primary: Color,
    /// Brand purple for accents and headers
    pub purple: Color,
    /// Dimmed purple for borders
    pub purple_dim: Color,
    /// Emerald accent for selections and highlights
    pub accent: Color,
    /// Popup/elevated surface background
    pub surface: Color,
    /// App header background
    pub header_bg: Color,
    /// App header foreground (text on header)
    pub header_fg: Color,
    /// Table header text color
    pub table_header_fg: Color,
    /// Table header background
    pub table_header_bg: Color,
    /// Selected row background
    pub selected_bg: Color,
    /// Visual selection (marked) background
    pub marked_bg: Color,
    /// Alternating row background
    pub alt_row_bg: Color,
    /// Active/unpaused DAG indicator
    pub dag_active: Color,

    // Semantic colors (Airflow states) - same across themes
    pub state_success: Color,
    pub state_failed: Color,
    pub state_running: Color,
    pub state_queued: Color,
    pub state_up_for_retry: Color,
    pub state_up_for_reschedule: Color,
    pub state_skipped: Color,
    pub state_upstream_failed: Color,

    // -- Pre-built styles --
    /// Default style for most content
    pub default_style: Style,
    /// Border style
    pub border_style: Style,
    /// Panel title style
    pub title_style: Style,
    /// Table header row style
    pub table_header_style: Style,
    /// Selected/highlighted row style
    pub selected_row_style: Style,
    /// Visual selection (marked) row style
    pub marked_style: Style,
    /// Alternating row style
    pub alt_row_style: Style,
    /// Surface style for popups
    pub surface_style: Style,
    /// Default button style
    pub button_default: Style,
    /// Selected button style
    pub button_selected: Style,
    /// Default border color
    pub border_default: Color,
    /// Selected border color
    pub border_selected: Color,
}

impl Theme {
    /// Dark theme preset - the original flowrs color scheme.
    #[allow(clippy::similar_names)]
    pub fn dark() -> Self {
        // Color palette
        let text_primary = Color::Rgb(220, 220, 230); // #DCDCE6
        let purple = Color::Rgb(138, 118, 255); // #8A76FF
        let purple_dim = Color::Rgb(90, 80, 160); // #5A50A0
        let accent = Color::Rgb(0, 220, 160); // #00DCA0
        let surface = Color::Rgb(50, 55, 70); // #323746
        let header_bg = Color::Rgb(138, 118, 255); // #8A76FF
        let header_fg = Color::Rgb(20, 20, 30); // #14141E
        let table_header_fg = Color::Rgb(200, 200, 210); // #C8C8D2
        let table_header_bg = Color::Rgb(45, 50, 65); // #2D3241
        let selected_bg = Color::Rgb(0, 80, 60); // #00503C
        let marked_bg = Color::Rgb(80, 50, 120); // #503278
        let alt_row_bg = Color::Rgb(30, 32, 42); // #1E202A

        Self::build(
            text_primary,
            purple,
            purple_dim,
            accent,
            surface,
            header_bg,
            header_fg,
            table_header_fg,
            table_header_bg,
            selected_bg,
            marked_bg,
            alt_row_bg,
        )
    }

    /// Light theme preset - designed for terminals with light backgrounds.
    #[allow(clippy::similar_names)]
    pub fn light() -> Self {
        let text_primary = Color::Rgb(46, 46, 58); // #2E2E3A
        let purple = Color::Rgb(110, 86, 230); // #6E56E6
        let purple_dim = Color::Rgb(155, 143, 204); // #9B8FCC
        let accent = Color::Rgb(0, 140, 100); // #008C64
        let surface = Color::Rgb(240, 240, 245); // #F0F0F5
        let header_bg = Color::Rgb(110, 86, 230); // #6E56E6
        let header_fg = Color::Rgb(255, 255, 255); // #FFFFFF
        let table_header_fg = Color::Rgb(58, 58, 74); // #3A3A4A
        let table_header_bg = Color::Rgb(232, 232, 240); // #E8E8F0
        let selected_bg = Color::Rgb(179, 240, 224); // #B3F0E0
        let marked_bg = Color::Rgb(216, 200, 255); // #D8C8FF
        let alt_row_bg = Color::Rgb(245, 245, 250); // #F5F5FA

        Self::build(
            text_primary,
            purple,
            purple_dim,
            accent,
            surface,
            header_bg,
            header_fg,
            table_header_fg,
            table_header_bg,
            selected_bg,
            marked_bg,
            alt_row_bg,
        )
    }

    /// Catppuccin Latte - the lightest Catppuccin flavor.
    #[allow(clippy::similar_names)]
    pub fn catppuccin_latte() -> Self {
        let text_primary = Color::Rgb(76, 79, 105); // Text #4c4f69
        let purple = Color::Rgb(136, 57, 239); // Mauve #8839ef
        let purple_dim = Color::Rgb(124, 127, 147); // Overlay2 #7c7f93
        let accent = Color::Rgb(23, 146, 153); // Teal #179299
        let surface = Color::Rgb(204, 208, 218); // Surface0 #ccd0da
        let header_bg = Color::Rgb(136, 57, 239); // Mauve #8839ef
        let header_fg = Color::Rgb(239, 241, 245); // Base #eff1f5
        let table_header_fg = Color::Rgb(92, 95, 119); // Subtext1 #5c5f77
        let table_header_bg = Color::Rgb(188, 192, 204); // Surface1 #bcc0cc
        let selected_bg = Color::Rgb(172, 176, 190); // Surface2 #acb0be
        let marked_bg = Color::Rgb(220, 224, 232); // Crust #dce0e8
        let alt_row_bg = Color::Rgb(230, 233, 239); // Mantle #e6e9ef

        Self::build(
            text_primary,
            purple,
            purple_dim,
            accent,
            surface,
            header_bg,
            header_fg,
            table_header_fg,
            table_header_bg,
            selected_bg,
            marked_bg,
            alt_row_bg,
        )
    }

    /// Catppuccin Frappé - subdued colors with a muted aesthetic.
    #[allow(clippy::similar_names)]
    pub fn catppuccin_frappe() -> Self {
        let text_primary = Color::Rgb(198, 208, 245); // Text #c6d0f5
        let purple = Color::Rgb(202, 158, 230); // Mauve #ca9ee6
        let purple_dim = Color::Rgb(148, 156, 187); // Overlay2 #949cbb
        let accent = Color::Rgb(129, 200, 190); // Teal #81c8be
        let surface = Color::Rgb(65, 69, 89); // Surface0 #414559
        let header_bg = Color::Rgb(202, 158, 230); // Mauve #ca9ee6
        let header_fg = Color::Rgb(48, 52, 70); // Base #303446
        let table_header_fg = Color::Rgb(181, 191, 226); // Subtext1 #b5bfe2
        let table_header_bg = Color::Rgb(81, 87, 109); // Surface1 #51576d
        let selected_bg = Color::Rgb(98, 104, 128); // Surface2 #626880
        let marked_bg = Color::Rgb(115, 121, 148); // Overlay0 #737994
        let alt_row_bg = Color::Rgb(41, 44, 60); // Mantle #292c3c

        Self::build(
            text_primary,
            purple,
            purple_dim,
            accent,
            surface,
            header_bg,
            header_fg,
            table_header_fg,
            table_header_bg,
            selected_bg,
            marked_bg,
            alt_row_bg,
        )
    }

    /// Catppuccin Macchiato - gentle colors for a soothing atmosphere.
    #[allow(clippy::similar_names)]
    pub fn catppuccin_macchiato() -> Self {
        let text_primary = Color::Rgb(202, 211, 245); // Text #cad3f5
        let purple = Color::Rgb(198, 160, 246); // Mauve #c6a0f6
        let purple_dim = Color::Rgb(147, 154, 183); // Overlay2 #939ab7
        let accent = Color::Rgb(139, 213, 202); // Teal #8bd5ca
        let surface = Color::Rgb(54, 58, 79); // Surface0 #363a4f
        let header_bg = Color::Rgb(198, 160, 246); // Mauve #c6a0f6
        let header_fg = Color::Rgb(36, 39, 58); // Base #24273a
        let table_header_fg = Color::Rgb(184, 192, 224); // Subtext1 #b8c0e0
        let table_header_bg = Color::Rgb(73, 77, 100); // Surface1 #494d64
        let selected_bg = Color::Rgb(91, 96, 120); // Surface2 #5b6078
        let marked_bg = Color::Rgb(110, 115, 141); // Overlay0 #6e738d
        let alt_row_bg = Color::Rgb(30, 32, 48); // Mantle #1e2030

        Self::build(
            text_primary,
            purple,
            purple_dim,
            accent,
            surface,
            header_bg,
            header_fg,
            table_header_fg,
            table_header_bg,
            selected_bg,
            marked_bg,
            alt_row_bg,
        )
    }

    /// Catppuccin Mocha - the darkest flavor with cozy, color-rich accents.
    #[allow(clippy::similar_names)]
    pub fn catppuccin_mocha() -> Self {
        let text_primary = Color::Rgb(205, 214, 244); // Text #cdd6f4
        let purple = Color::Rgb(203, 166, 247); // Mauve #cba6f7
        let purple_dim = Color::Rgb(147, 153, 178); // Overlay2 #9399b2
        let accent = Color::Rgb(148, 226, 213); // Teal #94e2d5
        let surface = Color::Rgb(49, 50, 68); // Surface0 #313244
        let header_bg = Color::Rgb(203, 166, 247); // Mauve #cba6f7
        let header_fg = Color::Rgb(30, 30, 46); // Base #1e1e2e
        let table_header_fg = Color::Rgb(186, 194, 222); // Subtext1 #bac2de
        let table_header_bg = Color::Rgb(69, 71, 90); // Surface1 #45475a
        let selected_bg = Color::Rgb(88, 91, 112); // Surface2 #585b70
        let marked_bg = Color::Rgb(108, 112, 134); // Overlay0 #6c7086
        let alt_row_bg = Color::Rgb(24, 24, 37); // Mantle #181825

        Self::build(
            text_primary,
            purple,
            purple_dim,
            accent,
            surface,
            header_bg,
            header_fg,
            table_header_fg,
            table_header_bg,
            selected_bg,
            marked_bg,
            alt_row_bg,
        )
    }

    /// Build a theme from its color palette, deriving all styles.
    #[allow(clippy::too_many_arguments, clippy::similar_names)]
    fn build(
        text_primary: Color,
        purple: Color,
        purple_dim: Color,
        accent: Color,
        surface: Color,
        header_bg: Color,
        header_fg: Color,
        table_header_fg: Color,
        table_header_bg: Color,
        selected_bg: Color,
        marked_bg: Color,
        alt_row_bg: Color,
    ) -> Self {
        // Airflow state colors are semantic and identical across themes
        let state_success = Color::Rgb(0, 153, 0); // #009900
        let state_failed = Color::Rgb(255, 107, 107); // #FF6B6B
        let state_running = Color::Rgb(155, 255, 155); // #9BFF9B
        let state_queued = Color::Rgb(128, 128, 128); // #808080
        let state_up_for_retry = Color::Rgb(255, 179, 71); // #FFB347
        let state_up_for_reschedule = Color::Rgb(111, 231, 219); // #6FE7DB
        let state_skipped = Color::Rgb(255, 142, 198); // #FF8EC6
        let state_upstream_failed = Color::Rgb(255, 165, 0); // #FFA500
        let dag_active = Color::Rgb(30, 144, 255); // #1E90FF

        Self {
            text_primary,
            purple,
            purple_dim,
            accent,
            surface,
            header_bg,
            header_fg,
            table_header_fg,
            table_header_bg,
            selected_bg,
            marked_bg,
            alt_row_bg,
            dag_active,
            state_success,
            state_failed,
            state_running,
            state_queued,
            state_up_for_retry,
            state_up_for_reschedule,
            state_skipped,
            state_upstream_failed,

            // Derive styles from colors
            default_style: Style::new().fg(text_primary),
            border_style: Style::new().fg(purple_dim),
            title_style: Style::new().fg(purple).add_modifier(Modifier::BOLD),
            table_header_style: Style::new()
                .fg(table_header_fg)
                .bg(table_header_bg)
                .add_modifier(Modifier::BOLD),
            selected_row_style: Style::new().bg(selected_bg),
            marked_style: Style::new().fg(text_primary).bg(marked_bg),
            alt_row_style: Style::new().fg(text_primary).bg(alt_row_bg),
            surface_style: Style::new().fg(text_primary).bg(surface),
            button_default: Style::new().fg(text_primary).bg(surface),
            button_selected: Style::new()
                .fg(accent)
                .bg(selected_bg)
                .add_modifier(Modifier::BOLD),
            border_default: purple_dim,
            border_selected: accent,
        }
    }
}

// =============================================================================
// GLOBAL THEME ACCESS
// =============================================================================

static THEME: OnceLock<Theme> = OnceLock::new();

/// Initialize the global theme. Must be called once at startup before rendering.
///
/// # Panics
/// Panics if called more than once.
pub fn init_theme(mode: ThemeMode) {
    let theme = match mode {
        ThemeMode::Auto => {
            if detect_light_terminal() {
                log::info!("Auto-detected light terminal, using light theme");
                Theme::light()
            } else {
                log::info!("Auto-detected dark terminal, using dark theme");
                Theme::dark()
            }
        }
        ThemeMode::Dark => {
            log::info!("Using dark theme (configured)");
            Theme::dark()
        }
        ThemeMode::Light => {
            log::info!("Using light theme (configured)");
            Theme::light()
        }
        ThemeMode::CatppuccinLatte => {
            log::info!("Using Catppuccin Latte theme (configured)");
            Theme::catppuccin_latte()
        }
        ThemeMode::CatppuccinFrappe => {
            log::info!("Using Catppuccin Frappé theme (configured)");
            Theme::catppuccin_frappe()
        }
        ThemeMode::CatppuccinMacchiato => {
            log::info!("Using Catppuccin Macchiato theme (configured)");
            Theme::catppuccin_macchiato()
        }
        ThemeMode::CatppuccinMocha => {
            log::info!("Using Catppuccin Mocha theme (configured)");
            Theme::catppuccin_mocha()
        }
    };
    THEME
        .set(theme)
        .expect("init_theme must only be called once");
}

/// Returns the active theme. Falls back to dark theme if not initialized.
pub fn theme() -> &'static Theme {
    THEME.get_or_init(Theme::dark)
}

/// Detect whether the terminal has a light background.
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
