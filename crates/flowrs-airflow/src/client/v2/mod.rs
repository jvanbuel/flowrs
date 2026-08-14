pub mod model;

mod dag;
mod dagrun;
mod dagstats;
mod log;
mod task;
mod taskinstance;

use reqwest::{Method, RequestBuilder, Response, Url};

use super::base::BaseClient;
use crate::error::Result;

/// API v2 client implementation (for Airflow v3, uses /api/v2 endpoint)
#[derive(Debug)]
pub struct V2Client {
    base: BaseClient,
}

impl V2Client {
    const API_VERSION: &'static str = "api/v2";

    pub const fn new(base: BaseClient) -> Self {
        Self { base }
    }

    pub(crate) async fn base_api(&self, method: Method, endpoint: &str) -> Result<RequestBuilder> {
        self.base
            .base_api(method, endpoint, Self::API_VERSION)
            .await
    }

    pub(crate) async fn execute(&self, request: RequestBuilder) -> Result<Response> {
        self.base.execute(request).await
    }

    /// Returns the base endpoint URL for this client
    pub const fn endpoint(&self) -> &Url {
        self.base.endpoint()
    }
}
