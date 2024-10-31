use crate::airflow::config::FlowrsConfig;
use crate::app::error::Result;
use crate::app::model::dagruns::DagRunModel;
use crate::app::model::dags::DagModel;

use super::model::{config::ConfigModel, taskinstances::TaskInstanceModel};

pub struct App {
    pub dags: DagModel,
    pub configs: ConfigModel,
    pub dagruns: DagRunModel,
    pub ticks: u32,
    pub active_panel: Panel,
    pub task_instances: TaskInstanceModel,
}

#[derive(Clone)]
pub enum Panel {
    Config,
    Dag,
    DAGRun,
    TaskInstance,
}

impl App {
    pub async fn new(config: FlowrsConfig) -> Result<App> {
        let servers = config.servers.unwrap().clone();
        Ok(App {
            dags: DagModel::new(),
            configs: ConfigModel::new(servers),
            dagruns: DagRunModel::new(),
            task_instances: TaskInstanceModel::new(),
            active_panel: Panel::Dag,
            ticks: 0,
        })
    }

    pub fn next_panel(&mut self) {
        match self.active_panel {
            Panel::Config => self.active_panel = Panel::Dag,
            Panel::Dag => self.active_panel = Panel::DAGRun,
            Panel::DAGRun => self.active_panel = Panel::TaskInstance,
            Panel::TaskInstance => (),
        }
    }

    pub fn previous_panel(&mut self) {
        match self.active_panel {
            Panel::Config => (),
            Panel::Dag => self.active_panel = Panel::Config,
            Panel::DAGRun => self.active_panel = Panel::Dag,
            Panel::TaskInstance => self.active_panel = Panel::DAGRun,
        }
    }
}
