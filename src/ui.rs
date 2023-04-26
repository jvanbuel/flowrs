use tui::{
    backend::Backend,
    widgets::{Block, Borders},
    Frame,
};

use crate::app::state::{App, Panel};

use self::{config::render_config_panel, dag::render_dag_panel};

pub mod config;
pub mod dag;
pub mod dagrun;
pub mod taskinstance;

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    match app.active_panel {
        Panel::Config => {
            render_config_panel(f, app);
        }
        Panel::DAG => render_dag_panel(f, app),
        Panel::DAGRun => {
            let size = f.size();
            let block = Block::default().title("DAG Runs").borders(Borders::ALL);
            f.render_widget(block, size);
        }
        Panel::Task => {
            let size = f.size();
            let block = Block::default().title("Tasks").borders(Borders::ALL);
            f.render_widget(block, size);
        }
    }
}
