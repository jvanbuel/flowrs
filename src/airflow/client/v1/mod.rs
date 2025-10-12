mod dag;
mod dagrun;
mod dagstats;
mod log;
mod taskinstance;

use anyhow::Result;
use reqwest::Method;

use super::base::BaseClient;

/// API v1 client implementation (for Airflow v2, uses /api/v1 endpoint)
#[derive(Debug, Clone)]
pub struct V1Client {
    base: BaseClient,
}

impl V1Client {
    const API_VERSION: &'static str = "api/v1";

    /// Create a V1Client backed by the provided BaseClient.
    ///
    /// Returns a V1Client configured to use the given BaseClient for API requests.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use crate::airflow::client::v1::V1Client;
    /// use crate::airflow::client::base::BaseClient;
    ///
    /// // `base` should be an initialized BaseClient configured with authentication, base URL, etc.
    /// let base: BaseClient = /* initialized BaseClient */ unimplemented!();
    /// let client = V1Client::new(base);
    /// ```
    pub fn new(base: BaseClient) -> Self {
        Self { base }
    }

    /// Build a request targeting the V1 API for the given HTTP method and endpoint.
    ///
    /// The `endpoint` is appended to the client's API version prefix (`api/v1`) to form the request path.
    ///
    /// # Examples
    ///
    /// ```
    /// use reqwest::Method;
    /// // assume `client` is a V1Client already constructed
    /// let req = client.base_api(Method::GET, "dags/example_dag")?;
    /// ```
    ///
    /// # Returns
    ///
    /// A `Result` containing the configured `reqwest::RequestBuilder` on success, or an error otherwise.
    fn base_api(&self, method: Method, endpoint: &str) -> Result<reqwest::RequestBuilder> {
        self.base.base_api(method, endpoint, Self::API_VERSION)
    }
}