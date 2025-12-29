use anyhow::{Context, Result};
use log::{debug, info};
use reqwest::{Method, Url};
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use std::convert::TryFrom;
use std::time::Duration;

use crate::airflow::config::{AirflowAuth, AirflowConfig};
use crate::airflow::managed_services::conveyor::ConveyorClient;

/// Base HTTP client for Airflow API communication.
/// Handles authentication and provides base request building functionality.
#[derive(Debug, Clone)]
pub struct BaseClient {
    pub client: reqwest_middleware::ClientWithMiddleware,
    pub config: AirflowConfig,
}

impl BaseClient {
    pub fn new(config: AirflowConfig) -> Result<Self> {
        // Create the base reqwest client with timeout and TLS configuration
        let reqwest_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .use_rustls_tls()
            .build()?;
        
        // Configure retry policy: exponential backoff with 1 retry
        // This means each request will be attempted twice (initial + 1 retry)
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(1);
        
        // Wrap the client with retry middleware
        let client = ClientBuilder::new(reqwest_client)
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();
        
        Ok(Self { client, config })
    }

    /// Build a base request with authentication for the specified API version
    pub fn base_api(
        &self,
        method: Method,
        endpoint: &str,
        api_version: &str,
    ) -> Result<reqwest_middleware::RequestBuilder> {
        let base_url = Url::parse(&self.config.endpoint)?;
        let url = base_url.join(format!("{api_version}/{endpoint}").as_str())?;
        debug!("🔗 Request URL: {url}");

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
                        .context("Failed to run token helper command")?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        return Err(anyhow::anyhow!(
                            "Token helper command failed with exit code {:?}\nstdout: {}\nstderr: {}",
                            output.status.code(),
                            stdout,
                            stderr
                        ));
                    }

                    let token = String::from_utf8(output.stdout)
                        .context("Token helper returned invalid UTF-8")?
                        .trim()
                        .replace('"', "");
                    Ok(self.client.request(method, url).bearer_auth(token))
                } else {
                    if let Some(token) = &token.token {
                        return Ok(self.client.request(method, url).bearer_auth(token.trim()));
                    }
                    Err(anyhow::anyhow!("Token not found"))
                }
            }
            AirflowAuth::Conveyor => {
                info!("🔑 Conveyor Auth");
                let token: String = ConveyorClient::get_token()?;
                Ok(self.client.request(method, url).bearer_auth(token))
            }
            AirflowAuth::Mwaa(auth) => {
                info!("🔑 MWAA Auth: {}", auth.environment_name);
                Ok(self
                    .client
                    .request(method, url)
                    .header("Cookie", format!("session={}", auth.session_cookie)))
            }
            AirflowAuth::Astronomer(auth) => {
                info!("🔑 Astronomer Auth");
                Ok(self
                    .client
                    .request(method, url)
                    .bearer_auth(&auth.api_token))
            }
        }
    }
}

impl TryFrom<&AirflowConfig> for BaseClient {
    type Error = anyhow::Error;

    fn try_from(config: &AirflowConfig) -> Result<Self, Self::Error> {
        Self::new(config.clone())
    }
}
