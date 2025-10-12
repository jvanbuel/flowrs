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

/// Create an Airflow client appropriate for the provided configuration version.
///
/// Returns an `Arc`-wrapped implementation of `AirflowClient` selected from the
/// configured Airflow API version: `AirflowVersion::V2` yields a `V1Client` (uses API v1),
/// and `AirflowVersion::V3` yields a `V2Client` (uses API v2).
///
/// # Examples
///
/// ```no_run
/// use std::sync::Arc;
/// use crate::airflow::config::{AirflowConfig, AirflowVersion};
/// use crate::airflow::create_client;
/// use crate::airflow::traits::AirflowClient;
///
/// // Construct `config` appropriately for your environment.
/// let config = AirflowConfig { version: AirflowVersion::V2, ..Default::default() };
/// let client: Arc<dyn AirflowClient> = create_client(config).unwrap();
/// ```
pub fn create_client(config: AirflowConfig) -> Result<Arc<dyn AirflowClient>> {
    let base = BaseClient::new(config.clone())?;

    match config.version {
        AirflowVersion::V2 => Ok(Arc::new(V1Client::new(base))), // V2 uses API v1
        AirflowVersion::V3 => Ok(Arc::new(V2Client::new(base))), // V3 uses API v2
    }
}

#[cfg(test)]
mod tests {
    use crate::airflow::{
        client::BaseClient, managed_services::conveyor::get_conveyor_environment_servers,
    };
    use reqwest::Method;

    #[tokio::test]
    async fn test_base_api_conveyor() {
        let servers = get_conveyor_environment_servers().unwrap();
        let client = BaseClient::new(servers[0].clone()).unwrap();

        let response = client
            .base_api(Method::GET, "config", "api/v1")
            .unwrap()
            .send()
            .await;

        assert!(response.is_ok());
        assert!(response.unwrap().status().is_success());
    }
}