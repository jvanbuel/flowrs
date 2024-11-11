use std::sync::{Arc, Mutex};

use crate::app::state::{App, Panel};

use crate::app::model::dagruns::DagRunState;
use init_screen::render_init_screen;
use ratatui::widgets::{StatefulWidget, TableState, Widget};
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
            app.configs.render(f.area(), &mut f.buffer_mut());
        }
        _ => {}
        // Panel::Dag => f.render_stateful_widget(app.dags, f.area(), &mut app.dags.filtered.state),
        // Panel::DAGRun => f.render_stateful_widget(
        //     app.dagruns,
        //     f.area(),
        //     &mut DagRunState {
        //         table: app.dagruns.filtered.state.clone(),
        //         dag_code: app.dagruns.dag_code.vertical_scroll_state,
        //     },
        // ),
        // Panel::TaskInstance => f.render_stateful_widget(
        //     app.task_instances,
        //     f.area(),
        //     &mut app.task_instances.filtered.state,
        // ),
        // Panel::Logs => f.render_widget(app.logs, f.area()),
    }
}
