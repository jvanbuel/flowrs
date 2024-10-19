use std::{
    sync::{atomic::AtomicU32, Arc},
    vec,
};

use crate::airflow::model::{dagrun::DagRun, taskinstance::TaskInstance};

use crate::airflow::{client::AirFlowClient, config::FlowrsConfig};
use crate::app::error::Result;
use crate::app::model::dags::DagModel;
use crate::app::model::StatefulTable;

use super::model::config::ConfigModel;

pub struct FlowrsContext {
    pub client: AirFlowClient,
    pub ticks: AtomicU32,
}

impl FlowrsContext {
    pub fn new(client: AirFlowClient) -> Self {
        FlowrsContext {
            client,
            ticks: AtomicU32::new(0),
        }
    }
}

pub struct App {
    pub dags: DagModel,
    pub configs: ConfigModel,
    pub dagruns: StatefulTable<DagRun>,
    pub context: Arc<FlowrsContext>,
    pub active_panel: Panel,
    pub taskinstances: StatefulTable<TaskInstance>,
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
        let client = AirFlowClient::new(servers[0].clone())?;
        let context = Arc::new(FlowrsContext {
            client,
            ticks: AtomicU32::new(0),
        });
        Ok(App {
            dags: DagModel::new(context.clone()),
            configs: ConfigModel::new(context.clone(), servers),
            dagruns: StatefulTable::new(vec![]),
            context: context.clone(),
            active_panel: Panel::Dag,
            taskinstances: StatefulTable::new(vec![]),
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

    pub fn update_contexts(&mut self) {
        self.dags.context = self.context.clone();
        self.configs.context = self.context.clone();
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
