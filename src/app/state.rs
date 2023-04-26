use tui::widgets::TableState;

use crate::model::dag::Dag;

use super::{auth::Config, client::AirFlowClient};

pub struct App<'a> {
    pub dag_state: TableState,
    pub config_state: TableState,
    pub dags: Vec<Dag>,
    pub config: &'a Config,
    pub client: AirFlowClient<'a>,
    pub active_panel: Panel,
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
            dag_state: TableState::default(),
            config_state: TableState::default(),
            dags: daglist.dags,
            config: config,
            client,
            active_panel: Panel::DAG,
        }
    }

    pub async fn update_dags(&mut self) {
        let daglist = self.client.list_dags().await.unwrap();
        self.dags = daglist.dags;
    }

    pub async fn toggle_current_dag(&mut self) {
        let i = match self.dag_state.selected() {
            Some(i) => i,
            None => 0,
        };
        let dag_id = &self.dags[i].dag_id;
        let is_paused = self.dags[i].is_paused;

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

    pub fn next(&mut self) {
        let i = match self.dag_state.selected() {
            Some(i) => {
                if i >= self.dags.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.dag_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.dag_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.dags.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.dag_state.select(Some(i));
    }
}
