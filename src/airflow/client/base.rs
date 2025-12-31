use anyhow::{Context, Result};
use google_cloud_auth::credentials::{AccessTokenCredentials, Builder};
use log::{debug, info};
use reqwest::{Method, Url};
use std::convert::TryFrom;
use std::fmt;
use std::time::Duration;

use crate::airflow::config::{AirflowAuth, AirflowConfig};
use crate::airflow::managed_services::conveyor::ConveyorClient;

/// Base HTTP client for Airflow API communication.
/// Handles authentication and provides base request building functionality.
#[derive(Clone)]
pub struct BaseClient {
    pub client: reqwest::Client,
    pub config: AirflowConfig,
    gcp_credentials: Option<AccessTokenCredentials>,
}

impl fmt::Debug for BaseClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BaseClient")
            .field("client", &self.client)
            .field("config", &self.config)
            .field(
                "gcp_credentials",
                &self.gcp_credentials.as_ref().map(|_| "***"),
            )
            .finish()
    }
}

impl BaseClient {
    pub fn new(config: AirflowConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .use_rustls_tls()
            .build()?;

        let gcp_credentials = if matches!(config.auth, AirflowAuth::Composer(_)) {
            Some(
                Builder::default()
                    .build_access_token_credentials()
                    .context("Failed to build GCP credentials")?,
            )
        } else {
            None
        };

        Ok(Self {
            client,
            config,
            gcp_credentials,
        })
    }

    /// Build a base request with authentication for the specified API version
    pub async fn base_api(
        &self,
        method: Method,
        endpoint: &str,
        api_version: &str,
    ) -> Result<reqwest::RequestBuilder> {
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
            AirflowAuth::Token(token_source) => match token_source {
                crate::airflow::config::TokenSource::Command { cmd } => {
                    info!("🔑 Token Auth (command): {cmd}");
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
                }
                crate::airflow::config::TokenSource::Static { token } => {
                    info!("🔑 Token Auth (static)");
                    Ok(self.client.request(method, url).bearer_auth(token.trim()))
                }
            },
            AirflowAuth::Conveyor => {
                info!("🔑 Conveyor Auth");
                let token: String = ConveyorClient::get_token()?;
                Ok(self.client.request(method, url).bearer_auth(token))
            }
            AirflowAuth::Mwaa(auth) => {
                use crate::airflow::managed_services::mwaa::MwaaTokenType;
                info!("🔑 MWAA Auth: {}", auth.environment_name);
                match &auth.token {
                    MwaaTokenType::SessionCookie(cookie) => {
                        // Airflow 2.x: Use session cookie
                        Ok(self
                            .client
                            .request(method, url)
                            .header("Cookie", format!("session={cookie}")))
                    }
                    MwaaTokenType::JwtToken(token) => {
                        // Airflow 3.x: Use Bearer authentication
                        Ok(self.client.request(method, url).bearer_auth(token))
                    }
                }
            }
            AirflowAuth::Astronomer(auth) => {
                info!("🔑 Astronomer Auth");
                Ok(self
                    .client
                    .request(method, url)
                    .bearer_auth(&auth.api_token))
            }
            AirflowAuth::Composer(auth) => {
                info!(
                    "🔑 Composer Auth: {}/{}",
                    auth.project_id, auth.environment_name
                );
                let credentials = self
                    .gcp_credentials
                    .as_ref()
                    .expect("GCP credentials must be set for Composer auth");
                let token = credentials
                    .access_token()
                    .await
                    .context("Failed to get GCP access token")?;
                Ok(self.client.request(method, url).bearer_auth(token.token))
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
