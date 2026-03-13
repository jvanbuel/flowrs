pub mod auth;
pub mod base;
pub mod v1;
pub mod v2;

use anyhow::Result;

use crate::config::{AirflowConfig, AirflowVersion};

pub use base::BaseClient;
pub use v1::V1Client;
pub use v2::V2Client;

/// Enum wrapping the versioned API clients.
/// V1 is for Airflow v2 (uses /api/v1), V2 is for Airflow v3 (uses /api/v2).
pub enum AirflowApiClient {
    V1(V1Client),
    V2(V2Client),
}

/// Create an Airflow API client based on the configuration version
pub fn create_api_client(config: &AirflowConfig) -> Result<AirflowApiClient> {
    let base = BaseClient::new(config.clone())?;

    match config.version {
        AirflowVersion::V2 => Ok(AirflowApiClient::V1(V1Client::new(base))), // V2 uses API v1
        AirflowVersion::V3 => Ok(AirflowApiClient::V2(V2Client::new(base))), // V3 uses API v2
    }
}
