pub mod base;
pub mod v1;
pub mod v2;

use anyhow::Result;
use std::sync::Arc;

use crate::airflow::config::{AirflowConfig, AirflowVersion};
use crate::airflow::traits::AirflowClient;

pub use base::BaseClient;
pub use v1::V1Client;
pub use v2::V2Client;

/// Create an Airflow client based on the configuration version
pub fn create_client(config: &AirflowConfig) -> Result<Arc<dyn AirflowClient>> {
    let base = BaseClient::new(config.clone())?;

    match config.version {
        AirflowVersion::V2 => Ok(Arc::new(V1Client::new(base))), // V2 uses API v1
        AirflowVersion::V3 => Ok(Arc::new(V2Client::new(base))), // V3 uses API v2
    }
}
