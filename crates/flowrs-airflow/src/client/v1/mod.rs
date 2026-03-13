pub mod model;

mod dag;
mod dagrun;
mod dagstats;
pub mod log;
mod task;
mod taskinstance;

use anyhow::Result;
use reqwest::Method;

use super::base::BaseClient;

/// API v1 client implementation (for Airflow v2, uses /api/v1 endpoint)
#[derive(Debug)]
pub struct V1Client {
    pub base: BaseClient,
}

impl V1Client {
    const API_VERSION: &'static str = "api/v1";

    pub const fn new(base: BaseClient) -> Self {
        Self { base }
    }

    pub(crate) async fn base_api(
        &self,
        method: Method,
        endpoint: &str,
    ) -> Result<reqwest::RequestBuilder> {
        self.base
            .base_api(method, endpoint, Self::API_VERSION)
            .await
    }

    /// Returns the base endpoint URL for this client
    pub fn endpoint(&self) -> &str {
        &self.base.config.endpoint
    }
}
