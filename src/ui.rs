use ratatui::{
    backend::Backend,
    widgets::{Block, Borders},
    Frame,
};

use crate::app::state::{App, Panel};

use self::{config::render_config_panel, dag::render_dag_panel, dagrun::render_dagrun_panel};

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
        Panel::Task => {
            let size = f.size();
            let block = Block::default().title("Tasks").borders(Borders::ALL);
            f.render_widget(block, size);
        }
    }
}
