use tui::widgets::TableState;

use crate::model::dag::Dag;

use super::{auth::Config, client::AirFlowClient};

pub struct App<'a> {
    pub state: TableState,
    pub dags: Vec<Dag>,
    pub client: AirFlowClient<'a>,
}

impl<'a> App<'a> {
    pub async fn new(config: &'a Config) -> App<'a> {
        let client = AirFlowClient::new(&config.servers[1]);
        let daglist = client.list_dags().await.unwrap();
        App {
            state: TableState::default(),
            dags: daglist.dags,
            client,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.dags.len() - 1 {
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
                    self.dags.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}
