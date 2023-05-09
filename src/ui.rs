use ratatui::{backend::Backend, Frame};

use crate::app::state::{App, Panel};

use self::{
    config::render_config_panel, dag::render_dag_panel, dagrun::render_dagrun_panel,
    taskinstance::render_taskinstance_panel,
};

pub mod config;
pub mod dag;
pub mod dagrun;
pub mod taskinstance;

pub fn ui<B: Backend>(f: &mut Frame<'_, B>, app: &mut App) {
    match app.active_panel {
        Panel::Config => {
            render_config_panel(f, app);
        }
        Panel::DAG => render_dag_panel(f, app),
        Panel::DAGRun => render_dagrun_panel(f, app),
        Panel::TaskInstance => render_taskinstance_panel(f, app),
    }
}
