pub mod auth;
pub mod base;
pub mod v1;
pub mod v2;

use reqwest::Response;
use serde::de::DeserializeOwned;

use crate::error::{AirflowError, Result};

pub use base::BaseClient;
pub use v1::V1Client;
pub use v2::V2Client;

/// Deserialize a response body, keeping a snippet of it in the error on failure.
///
/// The body is read as text first rather than using `Response::json`, because some
/// deployments return payloads that do not match the documented schema (older v2.x
/// versions omit `dag_display_name`, for instance) and the raw body is what makes
/// those failures diagnosable.
pub(crate) async fn read_json<T: DeserializeOwned>(response: Response, context: &str) -> Result<T> {
    let body = response.text().await?;
    serde_json::from_str(&body).map_err(|e| AirflowError::decode(context, &body, e))
}
