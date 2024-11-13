use crate::airflow::config::FlowrsConfig;
use crate::app::error::Result;
use crate::app::model::dagruns::DagRunModel;
use crate::app::model::dags::DagModel;

use super::model::{config::ConfigModel, logs::LogModel, taskinstances::TaskInstanceModel};

pub struct App {
    pub config: FlowrsConfig,
    pub dags: DagModel,
    pub configs: ConfigModel,
    pub dagruns: DagRunModel,
    pub task_instances: TaskInstanceModel,
    pub logs: LogModel,
    pub ticks: u32,
    pub active_panel: Panel,
}

#[derive(Clone)]
pub enum Panel {
    Config,
    Dag,
    DAGRun,
    TaskInstance,
    Logs,
}

impl App {
    pub fn new(config: FlowrsConfig) -> Result<Self> {
        let servers = &config.clone().servers.unwrap();
        Ok(App {
            config,
            dags: DagModel::new(),
            configs: ConfigModel::new(servers.to_vec()),
            dagruns: DagRunModel::new(),
            task_instances: TaskInstanceModel::new(),
            logs: LogModel::new(),
            active_panel: Panel::Dag,
            ticks: 0,
        })
    }

    pub fn next_panel(&mut self) {
        match self.active_panel {
            Panel::Config => self.active_panel = Panel::Dag,
            Panel::Dag => self.active_panel = Panel::DAGRun,
            Panel::DAGRun => self.active_panel = Panel::TaskInstance,
            Panel::TaskInstance => self.active_panel = Panel::Logs,
            Panel::Logs => (),
        }
    }

    pub fn previous_panel(&mut self) {
        match self.active_panel {
            Panel::Config => (),
            Panel::Dag => self.active_panel = Panel::Config,
            Panel::DAGRun => self.active_panel = Panel::Dag,
            Panel::TaskInstance => self.active_panel = Panel::DAGRun,
            Panel::Logs => self.active_panel = Panel::TaskInstance,
        }
    }
}
