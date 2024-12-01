pub mod dagruns;
pub mod dags;
pub mod dagstats;
pub mod logs;
pub mod taskinstances;

use crate::app::error::Result;
use std::time::Duration;

use log::info;
use reqwest::{Method, Url};

use crate::airflow::config::{AirflowAuth, AirflowConfig};
use crate::app::error::{ConfigError, FlowrsError};

#[derive(Debug, Clone)]
pub struct AirFlowClient {
    pub client: reqwest::Client,
    pub config: AirflowConfig,
}

impl AirFlowClient {
    pub fn new(config: AirflowConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .cookie_store(true)
            .use_rustls_tls()
            .build()?;
        Ok(Self { client, config })
    }

    pub async fn initalize(mut self) -> Result<Self> {
        if let AirflowAuth::Session { initalized } = &self.config.auth {
            if !initalized {
                let aws_sdk_config =
                    aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
                let client = aws_sdk_mwaa::Client::new(&aws_sdk_config);
                let web_ui_token = client
                    .create_web_login_token()
                    .set_name(Some(self.config.name.clone()))
                    .send()
                    .await
                    .expect("Failed to get MWAA web login token");

                let mwaa_url = format!(
                    "https://{}/aws_mwaa/login",
                    web_ui_token
                        .clone()
                        .web_server_hostname
                        .expect("Failed to get MWAA web server hostname")
                );

                let _ = self
                    .client
                    .post(mwaa_url)
                    .form(&[("token", web_ui_token.clone().web_token.unwrap())])
                    .send()
                    .await?;

                self.config.auth = AirflowAuth::Session { initalized: true };
            }
        }
        Ok(self)
    }

    pub async fn base_api(
        &self,
        method: Method,
        endpoint: &str,
    ) -> Result<reqwest::RequestBuilder> {
        let base_url = Url::parse(&self.config.endpoint)?;
        let url = base_url.join(format!("api/v1/{endpoint}").as_str())?;

        match &self.config.auth {
            AirflowAuth::Basic(auth) => {
                info!("🔑 Basic Auth: {}", auth.username);
                Ok(self
                    .client
                    .request(method, url)
                    .basic_auth(&auth.username, Some(&auth.password)))
            }
            AirflowAuth::Token(token) => {
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
            AirflowAuth::Session { initalized: _ } => {
                info!("🔑 Session Auth");
                // TODO: pass client via Session enum?
                // Reference: https://docs.aws.amazon.com/mwaa/latest/userguide/access-mwaa-apache-airflow-rest-api.html#create-web-server-session-token

                Ok(self.client.request(method, url))
            }
        }
    }
}

impl From<AirflowConfig> for AirFlowClient {
    fn from(config: AirflowConfig) -> Self {
        Self::new(config).unwrap()
    }
}

#[cfg(test)]
mod tests {

    use reqwest::Method;

    use super::AirFlowClient;
    use crate::airflow::config::FlowrsConfig;

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

        let _ = client
            .base_api(Method::GET, "config")
            .await
            .unwrap()
            .send()
            .await;
    }
}
