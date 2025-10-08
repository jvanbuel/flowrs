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

    pub fn new(base: BaseClient) -> Self {
        Self { base }
    }

    fn base_api(&self, method: Method, endpoint: &str) -> Result<reqwest::RequestBuilder> {
        self.base.base_api(method, endpoint, Self::API_VERSION)
    }
}
