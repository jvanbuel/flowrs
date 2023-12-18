use crate::app::error::Result;
use std::time::Duration;

use log::debug;
use reqwest::{Method, Url};

use super::config::{AirflowAuth, AirflowConfig};

pub struct AirFlowClient {
    pub client: reqwest::Client,
    pub config: AirflowConfig,
}

impl AirFlowClient {
    pub fn new(config: AirflowConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .http1_title_case_headers()
            .timeout(Duration::from_secs(5))
            .build()?;
        Ok(Self { client, config })
    }

    pub fn base_api(&self, method: Method, endpoint: &str) -> Result<reqwest::RequestBuilder> {
        let base_url = Url::parse(&self.config.endpoint)?;
        let url = base_url.join(format!("api/v1/{endpoint}").as_str())?;
        match &self.config.auth {
            AirflowAuth::BasicAuth(auth) => Ok(self
                .client
                .request(method, url)
                .basic_auth(auth.username.clone(), Some(auth.password.clone()))),
            AirflowAuth::TokenAuth(token) => {
                if let Some(cmd) = &token.cmd {
                    let output = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(cmd)
                        .output()
                        .expect("failed to execute process");
                    let token = String::from_utf8(output.stdout)?;
                    debug!("🔑 Token: {}", token);
                    Ok(self.client.request(method, url).bearer_auth(token))
                } else {
                    Ok(self
                        .client
                        .request(method, url)
                        .bearer_auth(token.token.clone()))
                }
            }
        }
    }
}
