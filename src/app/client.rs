use std::error::Error;

use reqwest::{Method, Url};

use super::auth::AirflowConfig;

pub struct AirFlowClient<'a> {
    pub client: reqwest::Client,
    pub config: &'a AirflowConfig,
}

impl<'a> AirFlowClient<'a> {
    pub fn new(config: &'a AirflowConfig) -> Self {
        let client = reqwest::Client::builder()
            .http1_title_case_headers()
            .build()
            .unwrap();
        Self { client, config }
    }

    pub fn base_api(
        &self,
        method: Method,
        endpoint: &str,
    ) -> Result<reqwest::RequestBuilder, Box<dyn Error>> {
        let base_url = Url::parse(&self.config.endpoint)?;
        let url = base_url.join(format!("api/v1/{endpoint}").as_str())?;
        match &self.config.token {
            Some(token) => Ok(self.client.request(method, url).bearer_auth(token)),
            None => Ok(self.client.request(method, url).basic_auth(
                self.config.username.clone().unwrap(),
                self.config.password.clone(),
            )),
        }
    }
}
