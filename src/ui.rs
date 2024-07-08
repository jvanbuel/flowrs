use crate::app::state::{App, Panel};

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
pub mod taskinstance;

pub const TIME_FORMAT: &str = "[year]-[month]-[day] [hour]:[minute]:[second]";

pub fn ui(f: &mut Frame, app: &mut App) {
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
