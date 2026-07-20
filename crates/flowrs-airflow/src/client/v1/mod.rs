pub mod model;

mod dag;
mod dagrun;
mod dagstats;
pub mod log;
mod task;
mod taskinstance;

use reqwest::{Method, RequestBuilder, Response, Url};

use super::base::BaseClient;
use crate::error::Result;

/// API v1 client implementation (for Airflow v2, uses /api/v1 endpoint)
#[derive(Debug)]
pub struct V1Client {
    base: BaseClient,
}

impl V1Client {
    const API_VERSION: &'static str = "api/v1";

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
