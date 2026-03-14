mod catppuccin;
mod dark;
mod light;

use ratatui::style::{Color, Modifier, Style};

use super::Theme;

pub(super) struct ThemePalette {
    pub(super) text_primary: Color,
    pub(super) purple: Color,
    pub(super) purple_dim: Color,
    pub(super) accent: Color,
    pub(super) surface: Color,
    pub(super) header_bg: Color,
    pub(super) header_fg: Color,
    pub(super) table_header_fg: Color,
    pub(super) table_header_bg: Color,
    pub(super) selected_bg: Color,
    pub(super) marked_bg: Color,
    pub(super) alt_row_bg: Color,
    pub(super) schedule_fg: Color,
    pub(super) dag_active: Color,
    pub(super) border_selected: Color,
    pub(super) state_success: Color,
    pub(super) state_failed: Color,
    pub(super) state_running: Color,
    pub(super) state_queued: Color,
    pub(super) state_up_for_retry: Color,
    pub(super) state_up_for_reschedule: Color,
    pub(super) state_skipped: Color,
    pub(super) state_upstream_failed: Color,
}

impl Theme {
    pub(super) fn build(p: &ThemePalette) -> Self {
        Self {
            text_primary: p.text_primary,
            purple: p.purple,
            purple_dim: p.purple_dim,
            accent: p.accent,
            surface: p.surface,
            header_bg: p.header_bg,
            header_fg: p.header_fg,
            table_header_fg: p.table_header_fg,
            table_header_bg: p.table_header_bg,
            selected_bg: p.selected_bg,
            marked_bg: p.marked_bg,
            alt_row_bg: p.alt_row_bg,
            dag_active: p.dag_active,
            schedule_fg: p.schedule_fg,
            state_success: p.state_success,
            state_failed: p.state_failed,
            state_running: p.state_running,
            state_queued: p.state_queued,
            state_up_for_retry: p.state_up_for_retry,
            state_up_for_reschedule: p.state_up_for_reschedule,
            state_skipped: p.state_skipped,
            state_upstream_failed: p.state_upstream_failed,
            default_style: Style::new().fg(p.text_primary),
            border_style: Style::new().fg(p.purple_dim),
            title_style: Style::new().fg(p.purple).add_modifier(Modifier::BOLD),
            table_header_style: Style::new()
                .fg(p.table_header_fg)
                .bg(p.table_header_bg)
                .add_modifier(Modifier::BOLD),
            selected_row_style: Style::new().bg(p.selected_bg),
            marked_style: Style::new().fg(p.text_primary).bg(p.marked_bg),
            alt_row_style: Style::new().fg(p.text_primary).bg(p.alt_row_bg),
            surface_style: Style::new().fg(p.text_primary).bg(p.surface),
            button_default: Style::new().fg(p.text_primary).bg(p.surface),
            button_selected: Style::new()
                .fg(p.accent)
                .bg(p.selected_bg)
                .add_modifier(Modifier::BOLD),
            border_default: p.purple_dim,
            border_selected: p.border_selected,
        }
    }
}
