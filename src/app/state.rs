use std::{
    sync::{atomic::AtomicU32, Arc, RwLock},
    vec,
};

use crate::airflow::model::{
    dag::Dag,
    dagrun::DagRun,
    taskinstance::{TaskInstance, TaskInstanceList},
};

use crate::airflow::{
    client::AirFlowClient,
    config::{AirflowConfig, FlowrsConfig},
};
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
            configs: ConfigModel::new(servers),
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

    pub async fn update_dags(&mut self) {
        match self.context.client.list_dags().await {
            Ok(daglist) => {
                self.dags.all = daglist.dags;
                self.filter_dags();
            }
            Err(e) => {
                eprintln!("Error fetching dags: {}", e);
            }
        }
    }

    pub async fn update_dagruns(&mut self) {
        let current_dag_id =
            &self.dags.filtered.items[self.dags.filtered.state.selected().unwrap()].dag_id;
        self.dagruns.items = self
            .context
            .client
            .list_dagruns(current_dag_id)
            .await
            .unwrap()
            .dag_runs;
    }
    // pub async fn update_all_dagruns(&mut self) {
    //     self.all_dagruns = self.client.list_all_dagruns().await.unwrap();
    // }

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

    pub fn filter_dags(&mut self) {
        let prefix = &self.filter.prefix;
        let dags = &self.dags.all;
        let filtered_dags = match prefix {
            Some(prefix) => dags
                .iter()
                .filter(|dag| dag.dag_id.contains(prefix))
                .cloned()
                .collect::<Vec<Dag>>(),
            None => dags.clone(),
        };
        self.dags.filtered.items = filtered_dags;
    }

    pub async fn clear_dagrun(&mut self) {
        let dag_id = &self.get_current_dag_id();
        let dag_run_id = &self.get_current_dagrun_id();
        self.context
            .client
            .clear_dagrun(dag_id, dag_run_id)
            .await
            .unwrap();
    }

    pub async fn update_task_instances(&mut self) {
        let dag_id = &self.get_current_dag_id();
        let dag_run_id = &self.get_current_dagrun_id();
        self.taskinstances.items = self
            .context
            .client
            .list_task_instances(dag_id, dag_run_id)
            .await
            .unwrap()
            .task_instances;
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
