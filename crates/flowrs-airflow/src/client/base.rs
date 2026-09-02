use log::{debug, warn};
use reqwest::{Method, RequestBuilder, Response, Url};
use std::fmt;
use std::time::Duration;

use super::auth::{create_auth_provider, AuthProvider};
use crate::config::AirflowConfig;
use crate::error::{AirflowError, Result, SNIPPET_LEN};

/// Base HTTP client for Airflow API communication.
/// Handles authentication and provides base request building functionality.
pub struct BaseClient {
    client: reqwest::Client,
    config: AirflowConfig,
    /// The endpoint parsed once at construction, so an unusable URL is reported
    /// when the client is created rather than on every request.
    endpoint: Url,
    auth_provider: Box<dyn AuthProvider>,
}

impl fmt::Debug for BaseClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BaseClient")
            .field("client", &self.client)
            .field("config", &self.config)
            .field("endpoint", &self.endpoint)
            .field("auth_provider", &"<AuthProvider>")
            .finish()
    }
}

impl BaseClient {
    pub fn new(config: AirflowConfig) -> Result<Self> {
        if config.insecure {
            warn!(
                "TLS certificate verification is disabled for server '{}'",
                config.name
            );
        }
        let mut endpoint = Url::parse(&config.endpoint)
            .map_err(|e| AirflowError::invalid_url(config.endpoint.clone(), e))?;
        // Ensure the base path ends with a slash so `base_api`'s relative `join`
        // extends the endpoint rather than replacing its final segment. Without
        // this, a reverse-proxy prefix such as `/airflow` would be dropped.
        if !endpoint.path().ends_with('/') {
            let with_slash = format!("{}/", endpoint.path());
            endpoint.set_path(&with_slash);
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .use_rustls_tls()
            .danger_accept_invalid_certs(config.insecure)
            .build()?;

        let auth_provider = create_auth_provider(&config.auth)?;

        Ok(Self {
            client,
            config,
            endpoint,
            auth_provider,
        })
    }

    /// The base URL of the Airflow deployment this client talks to.
    pub const fn endpoint(&self) -> &Url {
        &self.endpoint
    }

    pub const fn config(&self) -> &AirflowConfig {
        &self.config
    }

    /// Build a base request with authentication for the specified API version
    pub(crate) async fn base_api(
        &self,
        method: Method,
        endpoint: &str,
        api_version: &str,
    ) -> Result<RequestBuilder> {
        let path = format!("{api_version}/{endpoint}");
        let url = self
            .endpoint
            .join(&path)
            .map_err(|e| AirflowError::invalid_url(path, e))?;
        debug!("🔗 Request URL: {url}");

        let request = self.client.request(method, url);
        self.auth_provider.authenticate(request).await
    }

    /// Send a request, turning a non-success status into [`AirflowError::Status`].
    ///
    /// The request is built rather than sent through `RequestBuilder::send` so that
    /// the method and URL are available for the error message, and the body is read
    /// instead of calling `error_for_status`, which discards Airflow's `detail`.
    pub(crate) async fn execute(&self, request: RequestBuilder) -> Result<Response> {
        let request = request.build()?;
        let method = request.method().clone();
        let url = request.url().clone();

        let response = self.client.execute(request).await?;
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }

        // Only the first SNIPPET_LEN chars end up in the error, so never buffer
        // more than their UTF-8 upper bound: a misrouted request can come back
        // with an arbitrarily large HTML page rather than Airflow's small JSON.
        let body = read_body_prefix(response, SNIPPET_LEN * 4).await;
        Err(AirflowError::status(&method, &url, status, &body))
    }
}

/// Read at most `max_bytes` of the response body, stopping at the first read
/// error. Returns whatever was read so far (possibly nothing), lossily decoded.
async fn read_body_prefix(mut response: Response, max_bytes: usize) -> String {
    let mut buf = Vec::new();
    while buf.len() < max_bytes {
        match response.chunk().await {
            Ok(Some(chunk)) => {
                let take = chunk.len().min(max_bytes - buf.len());
                buf.extend_from_slice(&chunk[..take]);
            }
            Ok(None) | Err(_) => break,
        }
    }
    String::from_utf8_lossy(&buf).into_owned()
}

impl TryFrom<&AirflowConfig> for BaseClient {
    type Error = AirflowError;

    fn try_from(config: &AirflowConfig) -> Result<Self> {
        Self::new(config.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{AirflowAuth, TokenSource};

    fn config(endpoint: &str) -> AirflowConfig {
        AirflowConfig {
            name: "test".to_string(),
            endpoint: endpoint.to_string(),
            auth: AirflowAuth::Token(TokenSource::Static {
                token: "tok".to_string(),
            }),
            managed: None,
            version: crate::config::AirflowVersion::V3,
            timeout_secs: crate::config::default_timeout(),
            insecure: false,
        }
    }

    #[test]
    fn rejects_an_unusable_endpoint_at_construction() {
        let error = BaseClient::new(config("not a url")).expect_err("should reject");
        assert!(
            matches!(error, AirflowError::InvalidUrl { .. }),
            "got: {error:?}"
        );
    }

    #[test]
    fn accepts_a_valid_endpoint() {
        let client = BaseClient::new(config("http://localhost:8080")).expect("should accept");
        assert_eq!(client.endpoint().host_str(), Some("localhost"));
    }

    #[tokio::test]
    async fn read_body_prefix_is_bounded_and_keeps_a_short_body_intact() {
        let short = Response::from(http::Response::new("short body"));
        assert_eq!(read_body_prefix(short, 64).await, "short body");

        let big = "x".repeat(100_000);
        let response = Response::from(http::Response::new(big));
        let prefix = read_body_prefix(response, SNIPPET_LEN * 4).await;
        assert_eq!(prefix.len(), SNIPPET_LEN * 4);
    }

    #[test]
    fn preserves_a_reverse_proxy_prefix_when_building_requests() {
        let client =
            BaseClient::new(config("http://localhost:8080/airflow")).expect("should accept");
        let url = client.endpoint().join("api/v2/dags").expect("should join");
        assert_eq!(url.as_str(), "http://localhost:8080/airflow/api/v2/dags");
    }
}
