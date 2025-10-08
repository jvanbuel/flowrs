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

    pub fn new(base: BaseClient) -> Self {
        Self { base }
    }

    fn base_api(&self, method: Method, endpoint: &str) -> Result<reqwest::RequestBuilder> {
        self.base.base_api(method, endpoint, Self::API_VERSION)
    }
}
