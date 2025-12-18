# CI Testing Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Set up GitHub Actions CI with unit tests on PRs and integration tests on merge to main.

**Architecture:** Two separate workflow files - `ci.yml` for fast PR feedback (fmt, clippy, unit tests in parallel) and `integration.yml` for thorough testing against real Airflow instances (2.x and 3.x) on main branch merges.

**Tech Stack:** GitHub Actions, Docker (Airflow containers), Rust testing framework, async_trait for test clients.

---

## Task 1: Create PR CI Workflow

**Files:**
- Create: `.github/workflows/ci.yml`

**Step 1: Create the workflow file**

```yaml
name: CI

on:
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  fmt:
    name: Format
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all --check

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --all-targets --all-features -- -D warnings

  test:
    name: Unit Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --lib --bins
```

**Step 2: Verify the workflow syntax**

Run: `cat .github/workflows/ci.yml | head -50`
Expected: YAML file contents displayed without syntax errors

---

## Task 2: Create Integration Test Workflow

**Files:**
- Create: `.github/workflows/integration.yml`

**Step 1: Create the integration workflow file**

```yaml
name: Integration Tests

on:
  push:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  integration:
    name: Integration (${{ matrix.airflow_version }})
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        include:
          - airflow_version: "2.10.4"
            api_version: "v1"
          - airflow_version: "3.0.1"
            api_version: "v2"

    services:
      airflow:
        image: apache/airflow:${{ matrix.airflow_version }}
        env:
          AIRFLOW__CORE__LOAD_EXAMPLES: "true"
          AIRFLOW__DATABASE__SQL_ALCHEMY_CONN: sqlite:////tmp/airflow.db
          AIRFLOW__WEBSERVER__SECRET_KEY: test-secret-key
          AIRFLOW__API__AUTH_BACKENDS: airflow.api.auth.backend.basic_auth
          _AIRFLOW_DB_MIGRATE: "true"
          _AIRFLOW_WWW_USER_CREATE: "true"
          _AIRFLOW_WWW_USER_USERNAME: airflow
          _AIRFLOW_WWW_USER_PASSWORD: airflow
        ports:
          - 8080:8080
        options: >-
          --entrypoint /bin/bash
          --health-cmd "curl -f http://localhost:8080/health || exit 1"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 10
          --health-start-period 30s

    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable

      - uses: Swatinem/rust-cache@v2

      - name: Start Airflow
        run: |
          docker exec ${{ job.services.airflow.id }} bash -c "
            airflow db migrate &&
            airflow users create --username airflow --password airflow --firstname Test --lastname User --role Admin --email test@example.com &&
            airflow webserver --port 8080 &
            airflow scheduler &
            sleep 30
          "

      - name: Wait for Airflow API
        run: |
          timeout 120 bash -c '
            until curl -sf -u airflow:airflow http://localhost:8080/api/v1/health; do
              echo "Waiting for Airflow API..."
              sleep 5
            done
          '
          echo "Airflow API is ready"

      - name: Run integration tests
        env:
          TEST_AIRFLOW_URL: http://localhost:8080
          TEST_AIRFLOW_USERNAME: airflow
          TEST_AIRFLOW_PASSWORD: airflow
          TEST_API_VERSION: ${{ matrix.api_version }}
        run: cargo test --test '*' -- --test-threads=1
```

**Step 2: Verify the workflow syntax**

Run: `cat .github/workflows/integration.yml | head -80`
Expected: YAML file contents displayed

---

## Task 3: Create Test Utilities Module

**Files:**
- Create: `tests/common/mod.rs`

**Step 1: Create the tests directory structure**

Run: `mkdir -p tests/common`

**Step 2: Create the common test utilities module**

```rust
use std::env;
use std::sync::Arc;

use flowrs::airflow::client::{create_client, BaseClient, V1Client, V2Client};
use flowrs::airflow::config::{AirflowAuth, AirflowConfig, AirflowVersion, BasicAuth};
use flowrs::airflow::traits::AirflowClient;

/// Check if we should run tests for a specific API version
pub fn should_run_for_api_version(version: &str) -> bool {
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
        "v1" => AirflowVersion::V2, // Airflow 2.x uses API v1
        "v2" => AirflowVersion::V3, // Airflow 3.x uses API v2
        _ => AirflowVersion::V2,
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
```

**Step 3: Verify the file compiles**

Run: `cargo check --tests 2>&1 | tail -10`
Expected: No errors (warnings OK)

---

## Task 4: Create V1 API Integration Tests

**Files:**
- Create: `tests/v1_api_test.rs`

**Step 1: Create the V1 API test file**

```rust
mod common;

use common::{create_test_client, should_run_for_api_version};

#[tokio::test]
async fn test_v1_list_dags() {
    if !should_run_for_api_version("v1") {
        println!("Skipping V1 test - TEST_API_VERSION is not 'v1'");
        return;
    }

    let client = create_test_client().expect("Failed to create test client");
    let result = client.list_dags().await;

    assert!(result.is_ok(), "Failed to list DAGs: {:?}", result.err());

    let dag_list = result.unwrap();
    // Airflow with LOAD_EXAMPLES=true should have example DAGs
    assert!(
        !dag_list.dags.is_empty(),
        "Expected at least one DAG, got none"
    );
}

#[tokio::test]
async fn test_v1_dag_has_required_fields() {
    if !should_run_for_api_version("v1") {
        return;
    }

    let client = create_test_client().expect("Failed to create test client");
    let dag_list = client.list_dags().await.expect("Failed to list DAGs");

    if let Some(dag) = dag_list.dags.first() {
        // Verify DAG has required fields populated
        assert!(!dag.dag_id.is_empty(), "DAG ID should not be empty");
    }
}
```

**Step 2: Verify the test file compiles**

Run: `cargo check --tests 2>&1 | tail -10`
Expected: Compilation succeeds

---

## Task 5: Create V2 API Integration Tests

**Files:**
- Create: `tests/v2_api_test.rs`

**Step 1: Create the V2 API test file**

```rust
mod common;

use common::{create_test_client, should_run_for_api_version};

#[tokio::test]
async fn test_v2_list_dags() {
    if !should_run_for_api_version("v2") {
        println!("Skipping V2 test - TEST_API_VERSION is not 'v2'");
        return;
    }

    let client = create_test_client().expect("Failed to create test client");
    let result = client.list_dags().await;

    assert!(result.is_ok(), "Failed to list DAGs: {:?}", result.err());

    let dag_list = result.unwrap();
    // Airflow with LOAD_EXAMPLES=true should have example DAGs
    assert!(
        !dag_list.dags.is_empty(),
        "Expected at least one DAG, got none"
    );
}

#[tokio::test]
async fn test_v2_dag_has_required_fields() {
    if !should_run_for_api_version("v2") {
        return;
    }

    let client = create_test_client().expect("Failed to create test client");
    let dag_list = client.list_dags().await.expect("Failed to list DAGs");

    if let Some(dag) = dag_list.dags.first() {
        // Verify DAG has required fields populated
        assert!(!dag.dag_id.is_empty(), "DAG ID should not be empty");
    }
}
```

**Step 2: Verify the test file compiles**

Run: `cargo check --tests 2>&1 | tail -10`
Expected: Compilation succeeds

---

## Task 6: Verify Unit Tests Still Pass

**Files:**
- None (verification only)

**Step 1: Run unit tests to ensure nothing broke**

Run: `cargo test --lib --bins 2>&1 | tail -20`
Expected: Tests pass (the Conveyor test may fail due to external dependency - that's expected)

**Step 2: Verify integration tests compile (won't run without env vars)**

Run: `cargo test --test '*' --no-run 2>&1 | tail -10`
Expected: Compilation succeeds

---

## Task 7: Fix Conveyor Test for CI

**Files:**
- Modify: `src/airflow/managed_services/conveyor.rs`

**Step 1: Find the failing test**

Run: `grep -n "test_list_conveyor" src/airflow/managed_services/conveyor.rs`
Expected: Line number of the test

**Step 2: Add `#[ignore]` attribute to the test**

The test `test_list_conveyor_environments` requires the `conveyor` CLI to be installed. Add `#[ignore]` so it doesn't run in CI:

```rust
#[test]
#[ignore] // Requires conveyor CLI to be installed
fn test_list_conveyor_environments() {
```

**Step 3: Verify unit tests pass**

Run: `cargo test --lib --bins 2>&1 | tail -10`
Expected: All tests pass (ignored tests show as "ignored")

---

## Summary

After completing all tasks:

1. **PR workflow** (`.github/workflows/ci.yml`): Runs fmt, clippy, and unit tests on every PR
2. **Integration workflow** (`.github/workflows/integration.yml`): Runs integration tests against Airflow 2.x and 3.x on merge to main
3. **Test structure**: `tests/common/mod.rs` provides shared utilities, `tests/v1_api_test.rs` and `tests/v2_api_test.rs` contain version-specific tests
4. **Conveyor test**: Marked with `#[ignore]` to not block CI

The integration tests are minimal but provide a foundation. More tests can be added incrementally for DAG runs, task instances, and logs.
