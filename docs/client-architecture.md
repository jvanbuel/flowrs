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

### 3. V1Client and V2Client

Both implementations wrap a `BaseClient` and implement the `AirflowClient` trait:
- **V1Client** (`src/airflow/client/v1/*.rs`) - Uses `/api/v1` endpoint (for Airflow v2)
- **V2Client** (`src/airflow/client/v2/*.rs`) - Uses `/api/v2` endpoint (for Airflow v3)

Key differences between v1 and v2:
- v2 uses `-logical_date` for ordering DAG runs instead of v1's `-execution_date`
- v2 doesn't use `update_mask` query parameter for PATCH operations

### 4. Factory Function

`create_client(config: AirflowConfig)` creates the appropriate client based on the `version` field in the configuration:

```rust
pub fn create_client(config: AirflowConfig) -> Result<Arc<dyn AirflowClient>> {
    let base = BaseClient::new(config.clone())?;

    match config.version {
        AirflowVersion::V2 => Ok(Arc::new(V1Client::new(base))), // V2 uses API v1
        AirflowVersion::V3 => Ok(Arc::new(V2Client::new(base))), // V3 uses API v2
    }
}

// Usage:
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

The `BaseClient` provides backward compatibility through:
- `base_api_legacy()` method which automatically uses the configured API version
- Direct method implementations for common operations (list_dags, get_dag_runs, etc.)
- Existing tests and application code that use `BaseClient` directly continue to work without modification

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

1. Use `Arc<dyn AirflowClient>` as the client type in the `App` struct
2. Use `create_client()` factory function to create clients
3. The factory automatically selects V1Client or V2Client based on configuration

Example:
```rust
use crate::airflow::client::create_client;
use crate::airflow::traits::AirflowClient;

let client: Arc<dyn AirflowClient> = create_client(config)?;
let dags = client.list_dags().await?;
```
