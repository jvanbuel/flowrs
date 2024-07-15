use crate::app::error::Result;
use std::time::Duration;

use log::info;
use reqwest::{Method, Url};

use super::{
    config::{AirflowAuth, AirflowConfig},
    error::{ConfigError, FlowrsError},
};

#[derive(Debug, Clone)]
pub struct AirFlowClient {
    pub client: reqwest::Client,
    pub config: AirflowConfig,
}

impl AirFlowClient {
    pub fn new(config: AirflowConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            // .http1_title_case_headers()
            .timeout(Duration::from_secs(5))
            .use_rustls_tls()
            .build()?;
        Ok(Self { client, config })
    }

    pub fn base_api(&self, method: Method, endpoint: &str) -> Result<reqwest::RequestBuilder> {
        let base_url = Url::parse(&self.config.endpoint)?;
        let url = base_url.join(format!("api/v1/{endpoint}").as_str())?;

        match &self.config.auth {
            AirflowAuth::BasicAuth(auth) => {
                info!("🔑 Basic Auth: {}", auth.username);
                Ok(self
                    .client
                    .request(method, url)
                    .basic_auth(auth.username.clone(), Some(auth.password.clone())))
            }
            AirflowAuth::TokenAuth(token) => {
                info!("🔑 Token Auth: {:?}", token.cmd);
                if let Some(cmd) = &token.cmd {
                    let output = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(cmd)
                        .output()
                        .expect("failed to execute process");
                    // Be careful that there are no leading or trailing whitespace characters or quotation marks
                    let token = String::from_utf8(output.stdout)?.trim().replace('"', "");
                    Ok(self.client.request(method, url).bearer_auth(token))
                } else {
                    if let Some(token) = &token.token {
                        return Ok(self.client.request(method, url).bearer_auth(token.trim()));
                    }
                    Err(FlowrsError::ConfigError(ConfigError::NoTokenOrCmd))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use reqwest::Method;

    use crate::app::client::AirFlowClient;
    use crate::app::config::FlowrsConfig;

    const TEST_CONFIG: &str = r#"[[servers]]
        name = "conveyor-dev"
        endpoint = "https://app.conveyordata.com/environments/dev/airflow/"

        [servers.auth.TokenAuth]
        cmd = "conveyor auth get --quiet | jq -r .access_token"
        token = ""
        "#;

    #[tokio::test]
    async fn test_base_api_conveyor() {
        let config: FlowrsConfig = toml::from_str(str::trim(TEST_CONFIG)).unwrap();
        let client = AirFlowClient::new(config.servers.unwrap()[0].clone()).unwrap();

        let _ = client.base_api(Method::GET, "config").unwrap().send().await;
    }
}
