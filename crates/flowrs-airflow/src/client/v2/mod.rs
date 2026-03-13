pub mod model;

mod dag;
mod dagrun;
mod dagstats;
mod log;
mod task;
mod taskinstance;

use anyhow::Result;
use reqwest::Method;

use super::base::BaseClient;

/// API v2 client implementation (for Airflow v3, uses /api/v2 endpoint)
#[derive(Debug)]
pub struct V2Client {
    pub base: BaseClient,
}

impl V2Client {
    const API_VERSION: &'static str = "api/v2";

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
