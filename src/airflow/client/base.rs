use anyhow::Result;
use log::debug;
use reqwest::{Method, Url};
use std::convert::TryFrom;
use std::fmt;
use std::time::Duration;

use super::auth::{create_auth_provider, AuthProvider};
use crate::airflow::config::AirflowConfig;

/// Base HTTP client for Airflow API communication.
/// Handles authentication and provides base request building functionality.
pub struct BaseClient {
    pub client: reqwest::Client,
    pub config: AirflowConfig,
    auth_provider: Box<dyn AuthProvider>,
}

impl fmt::Debug for BaseClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BaseClient")
            .field("client", &self.client)
            .field("config", &self.config)
            .field("auth_provider", &"<AuthProvider>")
            .finish()
    }
}

impl BaseClient {
    pub fn new(config: AirflowConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .use_rustls_tls()
            .build()?;

        let auth_provider = create_auth_provider(&config.auth)?;

        Ok(Self {
            client,
            config,
            auth_provider,
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

        let request = self.client.request(method, url);
        self.auth_provider.authenticate(request).await
    }
}

impl TryFrom<&AirflowConfig> for BaseClient {
    type Error = anyhow::Error;

    fn try_from(config: &AirflowConfig) -> Result<Self, Self::Error> {
        Self::new(config.clone())
    }
}
