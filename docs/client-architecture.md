# Airflow Client Architecture

## Overview

The Flowrs codebase now supports both Airflow API v2 and v3 through a trait-based architecture. This allows the application to work with different Airflow versions while maintaining a consistent interface.

## Architecture Components

### 1. AirflowClient Trait (`src/airflow/traits.rs`)

The `AirflowClient` trait defines the common interface for all Airflow API operations:
- DAG operations (list, toggle, get code)
- DAG Run operations (list, mark, clear, trigger)
- Task Instance operations (list, mark, clear)
- Log operations (get task logs)
- Statistics operations (get dag stats)

### 2. BaseClient (`src/airflow/client/base.rs`)

The `BaseClient` handles:
- HTTP client initialization
- Authentication (Basic, Token, Conveyor)
- Base request building with proper API paths

It provides two methods:
- `base_api(method, endpoint, api_version)` - Build a request for a specific API version
- `base_api_legacy(method, endpoint)` - Build a request using the configured API version (for backward compatibility)

### 3. V2Client and V3Client

Both implementations wrap a `BaseClient` and implement the `AirflowClient` trait:
- **V2Client** (`src/airflow/client/v2.rs`) - Uses `/api/v1` endpoint
- **V3Client** (`src/airflow/client/v3.rs`) - Uses `/api/v2` endpoint

Key differences between v2 and v3:
- v3 uses `-logical_date` for ordering DAG runs instead of `-execution_date`
- v3 doesn't use `update_mask` query parameter for PATCH operations

### 4. Factory Function

`create_client(config: AirflowConfig)` creates the appropriate client based on the `version` field in the configuration:

```rust
let client = create_client(config)?;
// Returns Arc<dyn AirflowClient>
```

## Configuration

Set the Airflow version in your configuration file (`~/.flowrs`):

```toml
[[servers]]
name = "my-airflow-v2"
endpoint = "http://localhost:8080"
version = "V2"  # Default if not specified

[servers.auth.BasicAuth]
username = "airflow"
password = "airflow"

[[servers]]
name = "my-airflow-v3"
endpoint = "http://localhost:8081"
version = "V3"

[servers.auth.BasicAuth]
username = "airflow"
password = "airflow"
```

## Backward Compatibility

The existing code continues to work unchanged:
- `AirFlowClient` is now a type alias for `BaseClient`
- All legacy methods on `BaseClient` use `base_api_legacy()` which automatically uses the configured API version
- Existing tests and application code work without modification

## Usage Examples

### Using the Trait (Recommended for new code)

```rust
use crate::airflow::client::create_client;
use crate::airflow::traits::AirflowClient;

let client = create_client(config)?;
let dags = client.list_dags().await?;
```

### Using BaseClient Directly (Backward compatible)

```rust
use crate::airflow::client::BaseClient;

let client = BaseClient::new(config)?;
let dags = client.list_dags().await?;
```

## Updating the Application

To use the trait-based client in the application:

1. Replace `AirFlowClient` with `Arc<dyn AirflowClient>` in the `App` struct
2. Use `create_client()` instead of `AirFlowClient::new()`
3. Update method calls to use the trait methods

Example:
```rust
// Before
let client = AirFlowClient::new(config)?;

// After
let client = create_client(config)?;
```

## Testing

Each client implementation includes unit tests. To run them:

```bash
# Test v2 client (requires local Airflow v2)
cargo test --test v2

# Test v3 client (requires local Airflow v3)
cargo test --test v3 -- --ignored
```
