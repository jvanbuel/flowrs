use crate::app::state::{App, Panel};
use crate::ui::tabs::{TabBar, TAB_BAR_HEIGHT};
use crate::ui::theme::{HEADER_BG, HEADER_FG, TEXT_PRIMARY};
use init_screen::render_init_screen;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Widget};
use ratatui::Frame;
use std::sync::{Arc, Mutex};
use throbber_widgets_tui::Throbber;

pub mod common;
pub mod constants;
mod init_screen;
pub mod tabs;
pub mod theme;

pub const TIME_FORMAT: &str = "[year]-[month]-[day] [hour]:[minute]:[second]";

pub fn draw_ui(f: &mut Frame, app: &Arc<Mutex<App>>) {
    let mut app = app.lock().unwrap();
    if app.startup && app.ticks <= 10 {
        render_init_screen(f, app.ticks);
        return;
    }
    app.startup = false;

    // Split area vertically: header (1 line), tab bar (3 lines), panel (remaining)
    let [top_line, tab_area, panel_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(TAB_BAR_HEIGHT),
        Constraint::Min(0),
    ])
    .areas(f.area());

    // First, fill the entire top line with header background
    let header_bg_block = Block::default().style(Style::default().bg(HEADER_BG));
    f.render_widget(header_bg_block, top_line);

    // Split top line horizontally to align throbber to the right
    let [app_info, throbber_area] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(20)]).areas(top_line);

    // Render app name and version on the left, with breadcrumb - prominent purple header
    let version = env!("CARGO_PKG_VERSION");
    let breadcrumb = app.breadcrumb();

    let header_line = if let Some(ref crumb) = breadcrumb {
        Line::from(vec![
            Span::styled(
                format!(" Flowrs v{version} "),
                Style::default().fg(HEADER_FG).bg(HEADER_BG),
            ),
            Span::styled(
                format!(" {crumb} "),
                Style::default()
                    .fg(TEXT_PRIMARY)
                    .bg(HEADER_BG)
                    .add_modifier(Modifier::ITALIC),
            ),
        ])
    } else {
        Line::from(Span::styled(
            format!(" Flowrs v{version} "),
            Style::default().fg(HEADER_FG).bg(HEADER_BG),
        ))
    };

    f.render_widget(
        Paragraph::new(header_line).style(Style::default().bg(HEADER_BG)),
        app_info,
    );

    // Render throbber only when loading
    if app.loading {
        let throbber = Throbber::default()
            .label("Fetching data...")
            .style(Style::default().fg(HEADER_FG).bg(HEADER_BG))
            .throbber_set(throbber_widgets_tui::OGHAM_C);
        f.render_stateful_widget(throbber, throbber_area, &mut app.throbber_state);
    }

    // Render tab bar
    let active_tab_index = match app.active_panel {
        Panel::Config => 0,
        Panel::Dag => 1,
        Panel::DAGRun => 2,
        Panel::TaskInstance => 3,
        Panel::Logs => 4,
    };
    let tab_bar = TabBar::new(active_tab_index);
    f.render_widget(tab_bar, tab_area);

    // Only frame has the ability to set the cursor position, so we need to control the cursor filter from here
    // Not very elegant, and quite some duplication... Should be refactored
    match app.active_panel {
        Panel::Config => {
            app.configs.render(panel_area, f.buffer_mut());
            if app.configs.filter.is_enabled() {
                f.set_cursor_position(app.configs.filter.cursor.position);
            }
        }
        Panel::Dag => {
            app.dags.render(panel_area, f.buffer_mut());
            if app.dags.filter.is_enabled() {
                f.set_cursor_position(app.dags.filter.cursor.position);
            }
        }
        Panel::DAGRun => {
            app.dagruns.render(panel_area, f.buffer_mut());
            if app.dagruns.filter.is_enabled() {
                f.set_cursor_position(app.dagruns.filter.cursor.position);
            }
        }
        Panel::TaskInstance => {
            app.task_instances.render(panel_area, f.buffer_mut());
            if app.task_instances.filter.is_enabled() {
                f.set_cursor_position(app.task_instances.filter.cursor.position);
            }
        }
        Panel::Logs => app.logs.render(panel_area, f.buffer_mut()),
    }

    // Render global warning popup on top of all panels
    if let Some(warning_popup) = &app.warning_popup {
        warning_popup.render(panel_area, f.buffer_mut());
    }
}
