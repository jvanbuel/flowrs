use super::auth::AirflowConfig;

pub struct AirFlowClient<'a> {
    pub client: reqwest::Client,
    pub config: &'a AirflowConfig,
}

impl<'a> AirFlowClient<'a> {
    pub fn new(config: &'a AirflowConfig) -> Self {
        let client = reqwest::Client::new();
        Self { client, config }
    }
}
