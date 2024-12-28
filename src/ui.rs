use crate::app::state::{App, Panel};
use init_screen::render_init_screen;
use ratatui::widgets::Widget;
use ratatui::Frame;
use std::sync::{Arc, Mutex};

pub mod common;
pub mod constants;
mod init_screen;

pub const TIME_FORMAT: &str = "[year]-[month]-[day] [hour]:[minute]:[second]";

pub fn draw_ui(f: &mut Frame, app: &Arc<Mutex<App>>) {
    let mut app = app.lock().unwrap();
    if app.ticks <= 10 {
        render_init_screen(f, app.ticks);
        return;
    }
    app.loading = false;
    // Only frame has the ability to set the cursor position, so we need to control the cursor filter from here
    // Not very elegant, and quite some duplication... Should be refactored
    match app.active_panel {
        Panel::Config => {
            app.configs.render(f.area(), f.buffer_mut());
            if app.configs.filter.is_enabled() {
                f.set_cursor_position(app.configs.filter.cursor.position);
            }
        }
        Panel::Dag => {
            app.dags.render(f.area(), f.buffer_mut());
            if app.dags.filter.is_enabled() {
                f.set_cursor_position(app.dags.filter.cursor.position);
            }
        }
        Panel::DAGRun => {
            app.dagruns.render(f.area(), f.buffer_mut());
            if app.dagruns.filter.is_enabled() {
                f.set_cursor_position(app.dagruns.filter.cursor.position);
            }
        }
        Panel::TaskInstance => {
            app.task_instances.render(f.area(), f.buffer_mut());
            if app.task_instances.filter.is_enabled() {
                f.set_cursor_position(app.task_instances.filter.cursor.position);
            }
        }
        Panel::Logs => app.logs.render(f.area(), f.buffer_mut()),
    }
}
