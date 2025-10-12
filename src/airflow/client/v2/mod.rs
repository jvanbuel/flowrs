mod dag;
mod dagrun;
mod dagstats;
mod log;
mod taskinstance;

use anyhow::Result;
use reqwest::Method;

use super::base::BaseClient;

/// API v2 client implementation (for Airflow v3, uses /api/v2 endpoint)
#[derive(Debug, Clone)]
pub struct V2Client {
    base: BaseClient,
}

impl V2Client {
    const API_VERSION: &'static str = "api/v2";

    /// Creates a V2Client configured to use the provided BaseClient.
    ///
    /// # Returns
    ///
    /// A V2Client configured with the provided BaseClient.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use crate::airflow::client::v2::V2Client;
    /// use crate::airflow::client::base::BaseClient;
    ///
    /// // Construct a BaseClient by the appropriate means in your codebase.
    /// let base: BaseClient = unsafe { std::mem::zeroed() };
    /// let client = V2Client::new(base);
    /// ```
    pub fn new(base: BaseClient) -> Self {
        Self { base }
    }

    /// Build a request for an endpoint under the API v2 namespace.
    ///
    /// `method` is the HTTP method to use. `endpoint` is the path relative to the `api/v2` prefix (do not include the leading `/api/v2`).
    ///
    /// # Returns
    ///
    /// A `reqwest::RequestBuilder` configured for the given method and endpoint under `api/v2`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use reqwest::Method;
    /// # use crate::airflow::client::v2::V2Client;
    /// # use crate::airflow::client::base::BaseClient;
    /// # let base = BaseClient::new("http://example"); // pseudo-construct
    /// let client = V2Client::new(base);
    /// let req = client.base_api(Method::GET, "dags/my_dag");
    /// ```
    fn base_api(&self, method: Method, endpoint: &str) -> Result<reqwest::RequestBuilder> {
        self.base.base_api(method, endpoint, Self::API_VERSION)
    }
}