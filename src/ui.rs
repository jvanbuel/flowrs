use std::time::Duration;

use crate::app::state::{App, Panel};

use init_screen::render_init_screen;
use ratatui::Frame;

use self::{
    config::render_config_panel, dag::render_dag_panel, dagrun::render_dagrun_panel,
    taskinstance::render_taskinstance_panel,
};

pub mod config;
pub mod constants;
pub mod dag;
pub mod dagrun;
pub mod help;
mod init_screen;
pub mod taskinstance;

pub const TIME_FORMAT: &str = "[year]-[month]-[day] [hour]:[minute]:[second]";

pub fn ui(f: &mut Frame, app: &mut App) {
    if app.is_initializing {
        // Shouldn't happen here ==> initialization is done when config has been updated,
        // should happen in state loop that does API calls.
        // app.is_initializing = false;
        return render_init_screen(f, app);
    }
    match app.active_panel {
        Panel::Config => {
            render_config_panel(f, app);
        }
        Panel::Dag => render_dag_panel(f, app),
        Panel::DAGRun => render_dagrun_panel(f, app),
        Panel::TaskInstance => render_taskinstance_panel(f, app),
        Panel::Help => help::render_help_panel(f, app),
    }
}
