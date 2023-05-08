use ratatui::widgets::TableState;

use crate::model::{
    dag::Dag,
    dagrun::{DagRun, DagRunList},
};

use super::{
    auth::{AirflowConfig, Config},
    client::AirFlowClient,
    filter::Filter,
};

pub struct App {
    pub dags: StatefulTable<Dag>,
    pub configs: StatefulTable<AirflowConfig>,
    pub dagruns: StatefulTable<DagRun>,
    pub all_dagruns: DagRunList,
    pub active_config: Config,
    pub client: AirFlowClient,
    pub active_panel: Panel,
    pub filter: Filter,
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
    DAG,
    DAGRun,
    Task,
}

impl App {
    pub async fn new(config: Config) -> App {
        let server = config.servers[1].clone();
        let client = AirFlowClient::new(server);
        let daglist = client.list_dags().await.unwrap();
        let dagruns = client.list_all_dagruns().await.unwrap();
        App {
            dags: StatefulTable::new(daglist.dags),
            configs: StatefulTable::new(config.servers.clone()),
            dagruns: StatefulTable::new(vec![]),
            all_dagruns: dagruns,
            active_config: config,
            client,
            active_panel: Panel::DAG,
            filter: Filter::new(),
        }
    }

    pub async fn update_dags(&mut self) {
        let daglist = self.client.list_dags().await.unwrap();
        self.dags.items = daglist.dags;
    }

    pub async fn toggle_current_dag(&mut self) {
        let i = self.dags.state.selected().unwrap_or(0);
        let dag_id = &self.dags.items[i].dag_id;
        let is_paused = self.dags.items[i].is_paused;

        self.client.toggle_dag(dag_id, is_paused).await.unwrap();
    }

    pub async fn update_dagruns(&mut self) {
        let current_dag_id = &self.dags.items[self.dags.state.selected().unwrap()].dag_id;
        self.dagruns.items = self
            .client
            .list_dagruns(current_dag_id)
            .await
            .unwrap()
            .dag_runs;
    }
    pub async fn update_all_dagruns(&mut self) {
        self.all_dagruns = self.client.list_all_dagruns().await.unwrap();
    }

    pub fn next_panel(&mut self) {
        match self.active_panel {
            Panel::Config => self.active_panel = Panel::DAG,
            Panel::DAG => self.active_panel = Panel::DAGRun,
            Panel::DAGRun => self.active_panel = Panel::Task,
            Panel::Task => (),
        }
    }

    pub fn previous_panel(&mut self) {
        match self.active_panel {
            Panel::Config => (),
            Panel::DAG => self.active_panel = Panel::Config,
            Panel::DAGRun => self.active_panel = Panel::DAG,
            Panel::Task => self.active_panel = Panel::DAGRun,
        }
    }

    pub fn toggle_search(&mut self) {
        self.filter.toggle();
    }
}
