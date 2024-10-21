use std::vec;

use crate::airflow::model::{dagrun::DagRun, taskinstance::TaskInstance};

use crate::airflow::config::FlowrsConfig;
use crate::app::error::Result;
use crate::app::model::dags::DagModel;
use crate::app::model::StatefulTable;

use super::{model::config::ConfigModel, worker::WorkerMessage};
use tokio::sync::mpsc::Sender;

pub struct App {
    pub dags: DagModel,
    pub configs: ConfigModel,
    pub dagruns: StatefulTable<DagRun>,
    pub ticks: u32,
    pub active_panel: Panel,
    pub taskinstances: StatefulTable<TaskInstance>,
    pub tx_worker: Option<Sender<WorkerMessage>>,
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
            dagruns: StatefulTable::new(vec![]),
            taskinstances: StatefulTable::new(vec![]),
            active_panel: Panel::Dag,
            ticks: 0,
            tx_worker: None,
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

    pub fn register_worker(&mut self, tx_worker: Sender<WorkerMessage>) {
        self.tx_worker = Some(tx_worker.clone());
        self.dags.register_worker(tx_worker.clone());
        self.configs.register_worker(tx_worker);
    }
}
