use std::env;
use std::sync::Arc;

use flowrs_tui::airflow::client::create_client;
use flowrs_tui::airflow::config::{AirflowAuth, AirflowConfig, AirflowVersion, BasicAuth};
use flowrs_tui::airflow::traits::AirflowClient;

/// Check if we should run tests for a specific API version.
/// Returns false if `TEST_AIRFLOW_URL` is not set (required for all API tests).
pub fn should_run_for_api_version(version: &str) -> bool {
    // Skip if TEST_AIRFLOW_URL is not configured
    if env::var("TEST_AIRFLOW_URL").is_err() {
        return false;
    }

    let test_version = env::var("TEST_API_VERSION").unwrap_or_default();
    test_version.is_empty() || test_version == version
}

/// Create a test client from environment variables
pub fn create_test_client() -> anyhow::Result<Arc<dyn AirflowClient>> {
    let url = env::var("TEST_AIRFLOW_URL").expect("TEST_AIRFLOW_URL must be set");
    let username = env::var("TEST_AIRFLOW_USERNAME").unwrap_or_else(|_| "airflow".to_string());
    let password = env::var("TEST_AIRFLOW_PASSWORD").unwrap_or_else(|_| "airflow".to_string());
    let api_version = env::var("TEST_API_VERSION").unwrap_or_else(|_| "v1".to_string());

    let version = match api_version.as_str() {
        "v2" => AirflowVersion::V3, // Airflow 3.x uses API v2
        _ => AirflowVersion::V2,    // Airflow 2.x uses API v1 (default)
    };

    let config = AirflowConfig {
        name: "test".to_string(),
        endpoint: url,
        auth: AirflowAuth::Basic(BasicAuth { username, password }),
        managed: None,
        version,
        timeout_secs: 30,
    };

    create_client(&config)
}
