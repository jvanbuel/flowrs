//! Centralized theme module for Flowrs TUI.
//!
//! This module provides a comprehensive dark theme with:
//! - Color palette for text, accents, backgrounds, and semantic states
//! - Pre-built Style constants for common UI elements
//! - Consistent styling across tables, popups, buttons, and panels
//!
//! # Usage
//! Import the constants you need:
//! ```ignore
//! use crate::ui::theme::{BUTTON_SELECTED, BORDER_STYLE, ACCENT};
//! ```

use ratatui::style::{Color, Modifier, Style};

// =============================================================================
// COLOR PALETTE
// =============================================================================

// Text colors
pub const TEXT_PRIMARY: Color = Color::Rgb(220, 220, 230); // #DCDCE6 - Main text (light gray)

// Brand colors
pub const PURPLE: Color = Color::Rgb(138, 118, 255); // #8A76FF - Vibrant purple for accents
pub const PURPLE_DIM: Color = Color::Rgb(90, 80, 160); // #5A50A0 - Dimmed purple for borders

// Accent color - emerald for selections and highlights
pub const ACCENT: Color = Color::Rgb(0, 220, 160); // #00DCA0 - Bright emerald

// Background colors
pub const SURFACE: Color = Color::Rgb(50, 55, 70); // #323746 - Popup/elevated surface

// TUI Header - make it stand out
pub const HEADER_BG: Color = Color::Rgb(138, 118, 255); // #8A76FF - Purple background
pub const HEADER_FG: Color = Color::Rgb(20, 20, 30); // #14141E - Dark text on purple

// Table header - different from TUI header
pub const TABLE_HEADER_FG: Color = Color::Rgb(200, 200, 210); // #C8C8D2 - Bright text
pub const TABLE_HEADER_BG: Color = Color::Rgb(45, 50, 65); // #2D3241 - Subtle background

// Selection colors - very visible
pub const SELECTED_BG: Color = Color::Rgb(0, 80, 60); // #00503C - Teal/emerald selection
pub const MARKED_BG: Color = Color::Rgb(80, 50, 120); // #503278 - Purple for visual mode

// Semantic colors (Airflow states)
pub const STATE_SUCCESS: Color = Color::Rgb(80, 200, 120); // #50C878
pub const STATE_FAILED: Color = Color::Rgb(255, 107, 107); // #FF6B6B
pub const STATE_RUNNING: Color = Color::Rgb(34, 255, 170); // #22FFAA
pub const STATE_QUEUED: Color = Color::Rgb(128, 128, 128); // #808080
pub const STATE_UP_FOR_RETRY: Color = Color::Rgb(255, 179, 71); // #FFB347
pub const STATE_UP_FOR_RESCHEDULE: Color = Color::Rgb(111, 231, 219); // #6FE7DB
pub const STATE_SKIPPED: Color = Color::Rgb(255, 142, 198); // #FF8EC6
pub const STATE_UPSTREAM_FAILED: Color = Color::Rgb(255, 165, 0); // #FFA500

// DAG status colors
pub const DAG_ACTIVE: Color = Color::Rgb(30, 144, 255); // #1E90FF - Blue for active (unpaused) DAGs

// =============================================================================
// STYLES
// =============================================================================

// Default style for most content - light text, no background
pub const DEFAULT_STYLE: Style = Style {
    fg: Some(TEXT_PRIMARY),
    bg: None,
    underline_color: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

// Border style - use dimmed purple
pub const BORDER_STYLE: Style = Style {
    fg: Some(PURPLE_DIM),
    bg: None,
    underline_color: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

// Title style - purple accent for panel titles
pub const TITLE_STYLE: Style = Style {
    fg: Some(PURPLE),
    bg: None,
    underline_color: None,
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};

// Table header row - subtle but distinct
pub const TABLE_HEADER_STYLE: Style = Style {
    fg: Some(TABLE_HEADER_FG),
    bg: Some(TABLE_HEADER_BG),
    underline_color: None,
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};

// Selected row - bright emerald background (no fg to preserve cell colors)
pub const SELECTED_ROW_STYLE: Style = Style {
    fg: None,
    bg: Some(SELECTED_BG),
    underline_color: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

// Marked rows (visual selection mode) - purple background
pub const MARKED_STYLE: Style = Style {
    fg: Some(TEXT_PRIMARY),
    bg: Some(MARKED_BG),
    underline_color: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

// Alternating row - very subtle
pub const ALT_ROW_BG: Color = Color::Rgb(30, 32, 42); // #1E202A - Very subtle
pub const ALT_ROW_STYLE: Style = Style {
    fg: Some(TEXT_PRIMARY),
    bg: Some(ALT_ROW_BG),
    underline_color: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

// Surface style for popups
pub const SURFACE_STYLE: Style = Style {
    fg: Some(TEXT_PRIMARY),
    bg: Some(SURFACE),
    underline_color: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

// Button styles
pub const BUTTON_DEFAULT: Style = Style {
    fg: Some(TEXT_PRIMARY),
    bg: Some(SURFACE),
    underline_color: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

pub const BUTTON_SELECTED: Style = Style {
    fg: Some(ACCENT),
    bg: Some(SELECTED_BG),
    underline_color: None,
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};

// Border colors
pub const BORDER_DEFAULT: Color = PURPLE_DIM;
pub const BORDER_SELECTED: Color = ACCENT;
