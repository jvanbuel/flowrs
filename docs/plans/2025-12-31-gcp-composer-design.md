# GCP Composer Managed Service Design

## Overview

Add support for Google Cloud Composer as a managed service in flowrs, enabling auto-discovery of Composer environments across all accessible GCP projects.

## Dependencies

**New crate:**
```toml
google-cloud-auth = { version = "0.19", default-features = false, features = ["default-tls"] }
```

Uses the official Google Cloud Rust authentication library with Application Default Credentials.

## Module Structure

**New file:** `src/airflow/managed_services/composer.rs`

**Modified files:**
- `src/airflow/managed_services.rs` - Add `pub mod composer;`
- `src/airflow/config/mod.rs` - Add `AirflowAuth::Composer` variant, import, and expand logic
- `src/airflow/client/base.rs` - Handle `AirflowAuth::Composer` in request building

## Authentication

### Auth Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposerAuth {
    pub project_id: String,
    pub environment_name: String,
    pub access_token: String,
}
```

The access token is fetched during environment discovery using `google-cloud-auth` and stored in the struct. This matches the pattern used by MWAA (which stores a session cookie).

### OAuth Scope

```rust
const GCP_SCOPE: &str = "https://www.googleapis.com/auth/cloud-platform";
```

This single scope covers both Resource Manager and Composer APIs.

## ComposerClient Implementation

### Core Struct

```rust
pub struct ComposerClient {
    http_client: reqwest::Client,
    credentials: AccessTokenCredentials,
}
```

### API Endpoints

1. **List projects** (Resource Manager API):
   ```
   GET https://cloudresourcemanager.googleapis.com/v1/projects?filter=lifecycleState:ACTIVE
   ```

2. **List environments** (Composer API with wildcard location):
   ```
   GET https://composer.googleapis.com/v1/projects/{project}/locations/-/environments
   ```

### Methods

```rust
impl ComposerClient {
    pub async fn new() -> Result<Self>;
    pub async fn list_projects(&self) -> Result<Vec<Project>>;
    pub async fn list_environments(&self, project_id: &str) -> Result<Vec<ComposerEnvironment>>;
    pub async fn get_access_token(&self) -> Result<String>;
}
```

### Response Structs

```rust
pub struct Project {
    pub project_id: String,
    pub name: String,
    pub lifecycle_state: String,
}

pub struct ComposerEnvironment {
    pub name: String,           // projects/{project}/locations/{location}/environments/{name}
    pub config: EnvironmentConfig,
    pub state: String,          // RUNNING, CREATING, etc.
}

pub struct EnvironmentConfig {
    pub airflow_uri: String,    // The Airflow web UI URL
    pub software_config: SoftwareConfig,
}

pub struct SoftwareConfig {
    pub image_version: String,  // e.g., "composer-2.9.7-airflow-2.9.3"
}
```

## Environment Discovery Flow

**Main entry point:**
```rust
pub async fn get_composer_environment_servers() -> Result<Vec<AirflowConfig>>
```

**Flow:**
1. Create `ComposerClient` with default credentials
2. List all active projects via Resource Manager API
3. For each project, list environments using wildcard location (`-`)
4. Filter to only `RUNNING` environments
5. Parse Airflow version from `image_version` string
6. Convert each environment to `AirflowConfig`

**Version parsing:**
```rust
// image_version format: "composer-2.9.7-airflow-2.9.3"
fn parse_airflow_version(image_version: &str) -> AirflowVersion {
    if let Some(airflow_part) = image_version.split("-airflow-").nth(1) {
        if airflow_part.starts_with("3.") {
            return AirflowVersion::V3;
        }
    }
    AirflowVersion::V2  // Default to V2
}
```

**Resulting AirflowConfig:**
```rust
AirflowConfig {
    name: format!("{}/{}", project_id, environment_name),
    endpoint: airflow_uri,
    auth: AirflowAuth::Composer(ComposerAuth { project_id, environment_name, access_token }),
    managed: Some(ManagedService::Gcc),
    version: parse_airflow_version(&image_version),
    timeout_secs: 30,
}
```

## Error Handling

Lenient approach - errors are logged but not displayed to the user:
- Projects without Composer API enabled are skipped
- Projects with permission issues are skipped
- Projects with no environments are skipped
- Discovery continues with remaining projects

## Config Integration

**User configuration** (in `config.toml`):
```toml
managed_services = ["Gcc"]
```

No additional configuration needed - auto-discovers all accessible projects and environments.

**In `expand_managed_services`:**
```rust
ManagedService::Gcc => {
    let composer_servers = get_composer_environment_servers().await?;
    self.extend_servers(composer_servers);
}
```

## Request Authentication

**In `src/airflow/client/base.rs`:**
```rust
AirflowAuth::Composer(auth) => {
    info!("🔑 Composer Auth: {}/{}", auth.project_id, auth.environment_name);
    Ok(self.client.request(method, url).bearer_auth(&auth.access_token))
}
```
