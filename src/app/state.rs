use std::{
    sync::{atomic::AtomicU32, Arc},
    vec,
};

use crate::airflow::model::{
    dag::Dag,
    dagrun::DagRun,
    taskinstance::{TaskInstance, TaskInstanceList},
};

use crate::airflow::{client::AirFlowClient, config::FlowrsConfig};
use crate::app::error::Result;
use crate::app::model::dags::DagModel;
use crate::app::model::filter::Filter;
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
    pub active_config: FlowrsConfig,
    pub context: Arc<FlowrsContext>,
    pub active_panel: Panel,
    pub filter: Filter,
    pub taskinstances: StatefulTable<TaskInstance>,
    pub all_taskinstances: TaskInstanceList,
}

#[derive(Clone)]
pub enum Panel {
    Config,
    Dag,
    DAGRun,
    TaskInstance,
    Help,
}

impl App {
    pub async fn new(config: FlowrsConfig) -> Result<App> {
        let servers = config.clone().servers.unwrap().clone();
        let client = AirFlowClient::new(servers[0].clone())?;
        let context = Arc::new(FlowrsContext {
            client,
            ticks: AtomicU32::new(0),
        });
        Ok(App {
            dags: DagModel::new(context.clone()),
            configs: ConfigModel::new(context.clone(), servers),
            dagruns: StatefulTable::new(vec![]),
            active_config: config,
            context: context.clone(),
            active_panel: Panel::Dag,
            filter: Filter::new(),
            taskinstances: StatefulTable::new(vec![]),
            all_taskinstances: TaskInstanceList {
                task_instances: vec![],
                total_entries: 0,
            },
        })
    }

    pub fn next_panel(&mut self) {
        self.filter.reset();
        match self.active_panel {
            Panel::Config => self.active_panel = Panel::Dag,
            Panel::Dag => self.active_panel = Panel::DAGRun,
            Panel::DAGRun => self.active_panel = Panel::TaskInstance,
            Panel::TaskInstance => (),
            Panel::Help => (),
        }
    }

    pub fn update_contexts(&mut self) {
        self.dags.context = self.context.clone();
        self.configs.context = self.context.clone();
    }

    pub fn previous_panel(&mut self) {
        self.filter.reset();
        match self.active_panel {
            Panel::Config => (),
            Panel::Dag => self.active_panel = Panel::Config,
            Panel::DAGRun => self.active_panel = Panel::Dag,
            Panel::TaskInstance => self.active_panel = Panel::DAGRun,
            Panel::Help => (),
        }
    }

    pub fn toggle_search(&mut self) {
        self.filter.toggle();
    }

    fn get_current_dag_id(&self) -> String {
        self.dags
            .filtered
            .items
            .get(self.dags.filtered.state.selected().unwrap())
            .unwrap()
            .dag_id
            .clone()
    }

    fn get_current_dagrun_id(&self) -> String {
        self.dagruns
            .items
            .get(self.dagruns.state.selected().unwrap())
            .unwrap()
            .dag_run_id
            .clone()
    }
}
