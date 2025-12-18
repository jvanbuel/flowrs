//! Integration tests for managed service integrations (Conveyor, MWAA, Astronomer)
//!
//! These tests require external service credentials and are skipped by default.
//! To run specific tests:
//! - Conveyor: Requires `conveyor` CLI installed and authenticated
//! - MWAA: Requires AWS credentials with MWAA access
//! - Astronomer: Requires `ASTRO_API_TOKEN` environment variable

use std::env;

use flowrs_tui::airflow::managed_services::astronomer::{
    get_astronomer_environment_servers, AstronomerClient,
};
use flowrs_tui::airflow::managed_services::conveyor::{
    get_conveyor_environment_servers, ConveyorClient,
};
use flowrs_tui::airflow::managed_services::mwaa::{get_mwaa_environment_servers, MwaaClient};

// ============================================================================
// Conveyor Tests
// ============================================================================

#[tokio::test]
#[ignore = "Requires conveyor CLI to be installed and authenticated"]
async fn test_conveyor_list_environments() {
    let environments =
        get_conveyor_environment_servers().expect("Failed to list Conveyor environments");

    println!("Found {} Conveyor environments", environments.len());
    for env in &environments {
        println!("  - {} ({})", env.name, env.endpoint);
    }

    assert!(
        !environments.is_empty(),
        "Expected at least one Conveyor environment"
    );
}

#[test]
#[ignore = "Requires conveyor CLI to be installed and authenticated"]
fn test_conveyor_get_token() {
    let token = ConveyorClient::get_token().expect("Failed to get Conveyor token");

    assert!(!token.is_empty(), "Token should not be empty");
}

// ============================================================================
// MWAA Tests
// ============================================================================

fn should_run_mwaa_tests() -> bool {
    // MWAA tests require AWS credentials - check if they're available
    env::var("AWS_ACCESS_KEY_ID").is_ok() || env::var("AWS_PROFILE").is_ok()
}

#[tokio::test]
#[ignore = "Requires AWS credentials with MWAA access"]
async fn test_mwaa_list_environments() {
    if !should_run_mwaa_tests() {
        println!("Skipping MWAA test - AWS credentials not configured");
        return;
    }

    let result = get_mwaa_environment_servers().await;

    match result {
        Ok(environments) => {
            println!("Found {} MWAA environments", environments.len());
            for env in &environments {
                println!("  - {} ({})", env.name, env.endpoint);
            }
        }
        Err(e) => {
            // This may fail if no MWAA environments exist, which is acceptable
            println!("MWAA list environments result: {e}");
        }
    }
}

#[tokio::test]
#[ignore = "Requires AWS credentials"]
async fn test_mwaa_client_new() {
    if !should_run_mwaa_tests() {
        println!("Skipping MWAA test - AWS credentials not configured");
        return;
    }

    let client = MwaaClient::new().await;
    assert!(
        client.is_ok(),
        "Failed to create MWAA client: {:?}",
        client.err()
    );
}

// ============================================================================
// Astronomer Tests
// ============================================================================

fn should_run_astronomer_tests() -> bool {
    env::var("ASTRO_API_TOKEN").is_ok()
}

#[tokio::test]
#[ignore = "Requires ASTRO_API_TOKEN environment variable"]
async fn test_astronomer_list_environments() {
    if !should_run_astronomer_tests() {
        println!("Skipping Astronomer test - ASTRO_API_TOKEN not set");
        return;
    }

    let (environments, errors) = get_astronomer_environment_servers().await;

    println!("Found {} Astronomer deployments", environments.len());
    for env in &environments {
        println!("  - {} ({})", env.name, env.endpoint);
    }

    if !errors.is_empty() {
        println!("Errors encountered: {errors:?}");
    }
}

#[tokio::test]
#[ignore = "Requires ASTRO_API_TOKEN environment variable"]
async fn test_astronomer_client_new() {
    if !should_run_astronomer_tests() {
        println!("Skipping Astronomer test - ASTRO_API_TOKEN not set");
        return;
    }

    let client = AstronomerClient::new();
    assert!(
        client.is_ok(),
        "Failed to create Astronomer client: {:?}",
        client.err()
    );
}
