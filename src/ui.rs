use crate::app::{
    model::Model,
    state::{App, Panel},
};

use init_screen::render_init_screen;
use ratatui::Frame;

use self::{dagrun::render_dagrun_panel, taskinstance::render_taskinstance_panel};

pub mod constants;
pub mod dagrun;
mod init_screen;
pub mod taskinstance;

pub const TIME_FORMAT: &str = "[year]-[month]-[day] [hour]:[minute]:[second]";

pub fn draw_ui(f: &mut Frame, app: &mut App) {
    if app.context.ticks.load(std::sync::atomic::Ordering::Relaxed) == 0 {
        render_init_screen(f);
        return;
    }
    match app.active_panel {
        Panel::Config => {
            app.configs.view(f);
        }
        Panel::Dag => app.dags.view(f),
        Panel::DAGRun => render_dagrun_panel(f, app),
        Panel::TaskInstance => render_taskinstance_panel(f, app),
    }
}
