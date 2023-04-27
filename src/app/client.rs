use std::error::Error;

use reqwest::{Method, Url};

use super::auth::AirflowConfig;

pub struct AirFlowClient {
    pub client: reqwest::Client,
    pub config: AirflowConfig,
}

impl<'a> AirFlowClient {
    pub fn new(config: AirflowConfig) -> Self {
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
    ) -> Result<reqwest::RequestBuilder, Box<dyn Error + Send + Sync>> {
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
