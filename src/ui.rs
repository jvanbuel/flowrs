use std::sync::{Arc, Mutex};

use crate::app::{
    model::Model,
    state::{App, Panel},
};

use init_screen::render_init_screen;
use ratatui::Frame;

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
            // app.configs.view(f);
            f.render_stateful_widget(app.configs, f.area(), &mut app.configs.filtered.state);
        }
        Panel::Dag => app.dags.view(f),
        Panel::DAGRun => {
            f.render_stateful_widget(app.dagruns, f.area(), &mut app.dagruns.filtered.state)
        }
        Panel::TaskInstance => app.task_instances.view(f),
        Panel::Logs => app.logs.view(f),
    }
}
