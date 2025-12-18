# CI Testing Design

## Overview

Set up unit testing and integration testing in CI for the flowrs project.

- **Unit tests**: Run on every push to a PR
- **Integration tests**: Run when merging to main

## Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Test separation | `src/` for unit, `tests/` for integration | Cargo convention, clear separation |
| PR checks | Tests + Clippy + Format | Catches most issues without being slow |
| Integration approach | External Airflow services | Tests real API behavior |
| Airflow versions | 2.x (V1 API) + 3.x (V2 API) | Covers both client implementations |
| OS matrix | Linux only | Sufficient for API-focused code |

## Workflow Structure

### PR Workflow (`.github/workflows/ci.yml`)

**Triggers:** `pull_request` to `main`

**Jobs (run in parallel):**

1. **fmt** - Check code formatting
   - `cargo fmt --check`

2. **clippy** - Lint code
   - `cargo clippy -- -D warnings`

3. **test** - Run unit tests
   - `cargo test --lib --bins`

**Key features:**
- Uses `Swatinem/rust-cache@v2` for faster builds
- Treats clippy warnings as errors
- Only runs tests in `src/` (skips `tests/` directory)

### Integration Workflow (`.github/workflows/integration.yml`)

**Triggers:** `push` to `main`

**Matrix:**
| Airflow Version | API Version |
|-----------------|-------------|
| 2.10.4 | v1 |
| 3.0.1 | v2 |

**Jobs:**
1. Spin up Airflow container via GitHub Services
2. Wait for Airflow health endpoint
3. Run integration tests with environment variables:
   - `TEST_AIRFLOW_URL=http://localhost:8080`
   - `TEST_API_VERSION=v1` or `v2`

**Airflow container configuration:**
- LocalExecutor with SQLite
- Basic auth enabled for API access
- Port 8080 exposed

## Integration Test Structure

```
tests/
├── common/
│   └── mod.rs          # Shared test utilities
├── v1_api_test.rs      # Tests for Airflow 2.x V1 API
└── v2_api_test.rs      # Tests for Airflow 3.x V2 API
```

### Common Module (`tests/common/mod.rs`)

Provides:
- `create_test_client()` - Creates appropriate client based on `TEST_API_VERSION`
- Test fixtures and helper assertions
- Airflow authentication setup

### Test Pattern

Tests check the `TEST_API_VERSION` environment variable and skip if not applicable:

```rust
#[tokio::test]
async fn test_list_dags() {
    let api_version = env::var("TEST_API_VERSION").unwrap_or("v1".into());
    if api_version != "v1" {
        return; // Skip if not testing V1 API
    }

    let client = common::create_test_client().await;
    let dags = client.list_dags(None, None).await.unwrap();
    assert!(!dags.dags.is_empty());
}
```

## Files to Create

| File | Purpose |
|------|---------|
| `.github/workflows/ci.yml` | PR workflow |
| `.github/workflows/integration.yml` | Integration test workflow |
| `tests/common/mod.rs` | Shared test utilities |
| `tests/v1_api_test.rs` | V1 API integration tests |
| `tests/v2_api_test.rs` | V2 API integration tests |

## Known Issues

The existing test `test_list_conveyor_environments` fails in CI because it requires the `conveyor` CLI. Options:
1. Mark with `#[ignore]` for CI
2. Move to integration tests
3. Mock the external dependency

## Implementation Notes

- Use `cargo test --lib --bins` to run only unit tests (excludes `tests/` directory)
- Use `cargo test --test '*'` to run only integration tests (only `tests/` directory)
- GitHub Services handles container lifecycle automatically
- Tests read environment variables to determine which API version to test
