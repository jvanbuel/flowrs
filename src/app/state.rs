use ratatui::widgets::TableState;

use crate::model::dag::Dag;

use super::{
    auth::{AirflowConfig, Config},
    client::AirFlowClient,
};

pub struct App<'a> {
    pub dags: StatefulTable<Dag>,
    pub configs: StatefulTable<AirflowConfig>,
    pub active_config: &'a Config,
    pub client: AirFlowClient<'a>,
    pub active_panel: Panel,
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

impl<'a> App<'a> {
    pub async fn new(config: &'a Config) -> App<'a> {
        let client = AirFlowClient::new(&config.servers[1]);
        let daglist = client.list_dags().await.unwrap();
        App {
            dags: StatefulTable::new(daglist.dags),
            configs: StatefulTable::new(config.servers.clone()),
            active_config: config,
            client,
            active_panel: Panel::DAG,
        }
    }

    pub async fn update_dags(&mut self) {
        let daglist = self.client.list_dags().await.unwrap();
        self.dags.items = daglist.dags;
    }

    pub async fn toggle_current_dag(&mut self) {
        let i = match self.dags.state.selected() {
            Some(i) => i,
            None => 0,
        };
        let dag_id = &self.dags.items[i].dag_id;
        let is_paused = self.dags.items[i].is_paused;

        self.client.toggle_dag(dag_id, is_paused).await.unwrap();
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
}
