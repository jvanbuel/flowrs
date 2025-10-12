use crate::app::state::{App, Panel};
use crate::ui::constants::DEFAULT_STYLE;
use init_screen::render_init_screen;
use ratatui::layout::{Constraint, Layout};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Widget};
use ratatui::Frame;
use std::sync::{Arc, Mutex};
use throbber_widgets_tui::Throbber;

pub mod common;
pub mod constants;
mod init_screen;

pub const TIME_FORMAT: &str = "[year]-[month]-[day] [hour]:[minute]:[second]";

pub fn draw_ui(f: &mut Frame, app: &Arc<Mutex<App>>) {
    let mut app = app.lock().unwrap();
    if app.startup && app.ticks <= 10 {
        render_init_screen(f, app.ticks);
        return;
    }
    app.startup = false;

    // Always split area vertically to reserve top line for throbber
    let [top_line, panel_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(f.area());

    // Split top line horizontally to align throbber to the right
    let [app_info, throbber_area] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(20)]).areas(top_line);

    // Render app name and version on the left
    let version = env!("CARGO_PKG_VERSION");
    f.render_widget(
        Paragraph::new(Line::from(format!(" Flowrs v{version}"))).style(DEFAULT_STYLE),
        app_info,
    );

    // Render throbber only when loading
    if app.loading {
        let throbber = Throbber::default()
            .label("Fetching data...")
            .style(DEFAULT_STYLE)
            .throbber_set(throbber_widgets_tui::OGHAM_C);
        f.render_stateful_widget(throbber, throbber_area, &mut app.throbber_state);
    } else {
        // Render empty block with background when not loading
        let empty_block = Block::default().style(DEFAULT_STYLE);
        f.render_widget(empty_block, throbber_area);
    }

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
}
