use crate::app::state::{App, Panel};
use init_screen::render_init_screen;
use ratatui::widgets::Widget;
use ratatui::Frame;
use std::sync::{Arc, Mutex};

pub mod constants;
mod init_screen;

pub const TIME_FORMAT: &str = "[year]-[month]-[day] [hour]:[minute]:[second]";

pub fn draw_ui(f: &mut Frame, app: &Arc<Mutex<App>>) {
    let mut app = app.lock().unwrap();
    if app.ticks <= 10 {
        render_init_screen(f);
        return;
    }
    match app.active_panel {
        Panel::Config => {
            app.configs.render(f.area(), f.buffer_mut());
        }
        Panel::Dag => {
            app.dags.render(f.area(), f.buffer_mut());
        }
        Panel::DAGRun => {
            app.dagruns.render(f.area(), f.buffer_mut());
        }
        Panel::TaskInstance => {
            app.task_instances.render(f.area(), f.buffer_mut());
        }
        Panel::Logs => app.logs.render(f.area(), f.buffer_mut()),
    }
}
