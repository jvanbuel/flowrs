use ratatui::widgets::TableState;

use crate::model::{
    dag::{Dag, DagList},
    dagrun::{DagRun, DagRunList},
    taskinstance::{TaskInstance, TaskInstanceList},
};

use super::{
    client::AirFlowClient,
    config::{AirflowConfig, FlowrsConfig},
    filter::Filter,
};

pub struct App {
    pub all_dags: DagList,
    pub filtered_dags: StatefulTable<Dag>,
    pub configs: StatefulTable<AirflowConfig>,
    pub dagruns: StatefulTable<DagRun>,
    pub all_dagruns: DagRunList,
    pub active_config: FlowrsConfig,
    pub client: AirFlowClient,
    pub active_panel: Panel,
    pub filter: Filter,
    pub taskinstances: StatefulTable<TaskInstance>,
    pub all_taskinstances: TaskInstanceList,
    pub active_popup: bool,
}

#[derive(Clone)]
pub struct StatefulTable<T> {
    pub state: TableState,
    pub items: Vec<T>,
}

impl<T> StatefulTable<T> {
    pub fn new(items: Vec<T>) -> StatefulTable<T> {
        StatefulTable {
            state: TableState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

pub enum Panel {
    Config,
    Dag,
    DAGRun,
    TaskInstance,
}

impl App {
    pub async fn new(config: FlowrsConfig) -> App {
        let server = config.servers[1].clone();
        let client = AirFlowClient::new(server);
        let daglist = client.list_dags().await.unwrap();
        let dagruns = client.list_all_dagruns().await.unwrap();
        let taskinstances = client.list_all_taskinstances().await.unwrap();
        App {
            all_dags: daglist.clone(),
            filtered_dags: StatefulTable::new(daglist.dags),
            configs: StatefulTable::new(config.servers.clone()),
            dagruns: StatefulTable::new(vec![]),
            all_dagruns: dagruns,
            active_config: config,
            client,
            active_panel: Panel::Dag,
            filter: Filter::new(),
            taskinstances: StatefulTable::new(vec![]),
            all_taskinstances: taskinstances,
            active_popup: false,
        }
    }

    pub async fn update_dags(&mut self) {
        self.all_dags = self.client.list_dags().await.unwrap();
    }

    pub async fn toggle_current_dag(&mut self) {
        let i = self.filtered_dags.state.selected().unwrap_or(0);
        let dag_id = &self.filtered_dags.items[i].dag_id;
        let is_paused = self.filtered_dags.items[i].is_paused;

        self.client.toggle_dag(dag_id, is_paused).await.unwrap();
    }

    pub async fn update_dagruns(&mut self) {
        let current_dag_id =
            &self.filtered_dags.items[self.filtered_dags.state.selected().unwrap()].dag_id;
        self.dagruns.items = self
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
        }
    }

    pub fn previous_panel(&mut self) {
        self.filter.reset();
        match self.active_panel {
            Panel::Config => (),
            Panel::Dag => self.active_panel = Panel::Config,
            Panel::DAGRun => self.active_panel = Panel::Dag,
            Panel::TaskInstance => self.active_panel = Panel::DAGRun,
        }
    }

    pub fn toggle_search(&mut self) {
        self.filter.toggle();
    }

    pub fn filter_dags(&mut self) {
        let prefix = &self.filter.prefix;
        let dags = &self.all_dags.dags;
        let filtered_dags = match prefix {
            Some(prefix) => dags
                .iter()
                .filter(|dag| dag.dag_id.contains(prefix))
                .cloned()
                .collect::<Vec<Dag>>(),
            None => dags.clone(),
        };
        self.filtered_dags.items = filtered_dags;
    }

    pub async fn clear_dagrun(&mut self) {
        let dag_id = &self.get_current_dag_id();
        let dag_run_id = &self.get_current_dagrun_id();
        self.client.clear_dagrun(dag_id, dag_run_id).await.unwrap();
    }

    pub async fn update_task_instances(&mut self) {
        let dag_id = &self.get_current_dag_id();
        let dag_run_id = &self.get_current_dagrun_id();
        self.taskinstances.items = self
            .client
            .list_task_instances(dag_id, dag_run_id)
            .await
            .unwrap()
            .task_instances;
    }

    fn get_current_dag_id(&self) -> String {
        self.filtered_dags
            .items
            .get(self.filtered_dags.state.selected().unwrap())
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
