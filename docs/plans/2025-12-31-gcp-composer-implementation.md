# GCP Composer Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add GCP Composer as a managed service with auto-discovery of all environments across accessible projects.

**Architecture:** Use `google-cloud-auth` for Application Default Credentials, manual REST calls to Resource Manager and Composer APIs, store access token in auth struct for request authentication.

**Tech Stack:** google-cloud-auth, reqwest, serde, tokio

---

### Task 1: Add google-cloud-auth Dependency

**Files:**
- Modify: `Cargo.toml:20-58`

**Step 1: Add the dependency**

Add after line 36 (after `futures`):

```toml
google-cloud-auth = "0.19"
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: Compiles successfully (may take time to download)

---

### Task 2: Create ComposerAuth Struct and Add to AirflowAuth Enum

**Files:**
- Create: `src/airflow/managed_services/composer.rs`
- Modify: `src/airflow/managed_services.rs`
- Modify: `src/airflow/config/mod.rs:80-87`

**Step 1: Create the composer module with auth struct**

Create `src/airflow/managed_services/composer.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::fmt;

/// Composer authentication data including access token
#[derive(Clone, Serialize, Deserialize)]
pub struct ComposerAuth {
    pub project_id: String,
    pub environment_name: String,
    pub access_token: String,
}

impl fmt::Debug for ComposerAuth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ComposerAuth")
            .field("project_id", &self.project_id)
            .field("environment_name", &self.environment_name)
            .field("access_token", &"***redacted***")
            .finish()
    }
}
```

**Step 2: Export the module**

In `src/airflow/managed_services.rs`, add:

```rust
pub mod composer;
```

**Step 3: Add Composer variant to AirflowAuth enum**

In `src/airflow/config/mod.rs`, add import at line 15:

```rust
use super::managed_services::composer::ComposerAuth;
```

Then modify the `AirflowAuth` enum (lines 80-87) to add the Composer variant:

```rust
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum AirflowAuth {
    Basic(BasicAuth),
    Token(TokenCmd),
    Conveyor,
    Mwaa(super::managed_services::mwaa::MwaaAuth),
    Astronomer(super::managed_services::astronomer::AstronomerAuth),
    Composer(ComposerAuth),
}
```

**Step 4: Verify it compiles**

Run: `cargo check`
Expected: Compiles successfully

---

### Task 3: Add Composer Auth Handling in BaseClient

**Files:**
- Modify: `src/airflow/client/base.rs:90-97`

**Step 1: Add the Composer match arm**

After the `AirflowAuth::Astronomer` match arm (line 96), add:

```rust
            AirflowAuth::Composer(auth) => {
                info!(
                    "🔑 Composer Auth: {}/{}",
                    auth.project_id, auth.environment_name
                );
                Ok(self
                    .client
                    .request(method, url)
                    .bearer_auth(&auth.access_token))
            }
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: Compiles successfully

---

### Task 4: Add API Response Structs

**Files:**
- Modify: `src/airflow/managed_services/composer.rs`

**Step 1: Add response structs for GCP APIs**

Append to `src/airflow/managed_services/composer.rs`:

```rust

/// GCP Project from Resource Manager API
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub project_id: String,
    pub name: String,
    pub lifecycle_state: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectsResponse {
    pub projects: Option<Vec<Project>>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

/// Composer Environment from Composer API
#[derive(Debug, Clone, Deserialize)]
pub struct ComposerEnvironment {
    pub name: String,
    pub config: Option<EnvironmentConfig>,
    pub state: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentConfig {
    pub airflow_uri: Option<String>,
    pub software_config: Option<SoftwareConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareConfig {
    pub image_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnvironmentsResponse {
    pub environments: Option<Vec<ComposerEnvironment>>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: Compiles successfully

---

### Task 5: Implement ComposerClient

**Files:**
- Modify: `src/airflow/managed_services/composer.rs`

**Step 1: Add imports and constants**

At the top of `src/airflow/managed_services/composer.rs`, replace the imports with:

```rust
use anyhow::{Context, Result};
use google_cloud_auth::credentials::Builder;
use log::{debug, error, info};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;

const GCP_SCOPE: &str = "https://www.googleapis.com/auth/cloud-platform";
const RESOURCE_MANAGER_URL: &str = "https://cloudresourcemanager.googleapis.com/v1/projects";
const COMPOSER_API_URL: &str = "https://composer.googleapis.com/v1";
```

**Step 2: Add the ComposerClient struct and implementation**

Append after the response structs:

```rust

/// Composer client for managing authentication and environment discovery
pub struct ComposerClient {
    http_client: reqwest::Client,
}

impl ComposerClient {
    /// Creates a new Composer client
    pub fn new() -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { http_client })
    }

    /// Gets an access token using Application Default Credentials
    pub async fn get_access_token() -> Result<String> {
        let credentials = Builder::default()
            .with_scopes([GCP_SCOPE])
            .build_access_token_credentials()
            .context("Failed to build GCP credentials")?;

        let token = credentials
            .access_token()
            .await
            .context("Failed to get access token")?;

        Ok(token.token)
    }

    /// Lists all active projects the user has access to
    pub async fn list_projects(&self, access_token: &str) -> Result<Vec<Project>> {
        let mut all_projects = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let mut url = format!("{RESOURCE_MANAGER_URL}?filter=lifecycleState:ACTIVE");
            if let Some(token) = &page_token {
                url.push_str(&format!("&pageToken={token}"));
            }

            debug!("Fetching projects from: {url}");

            let response = self
                .http_client
                .get(&url)
                .header(AUTHORIZATION, format!("Bearer {access_token}"))
                .send()
                .await
                .context("Failed to list GCP projects")?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to list projects: HTTP {status} - {body}");
            }

            let projects_response: ProjectsResponse = response
                .json()
                .await
                .context("Failed to parse projects response")?;

            if let Some(projects) = projects_response.projects {
                all_projects.extend(projects);
            }

            match projects_response.next_page_token {
                Some(token) if !token.is_empty() => page_token = Some(token),
                _ => break,
            }
        }

        info!("Found {} GCP project(s)", all_projects.len());
        Ok(all_projects)
    }

    /// Lists all Composer environments in a project (using wildcard location)
    pub async fn list_environments(
        &self,
        project_id: &str,
        access_token: &str,
    ) -> Result<Vec<ComposerEnvironment>> {
        let mut all_environments = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let mut url =
                format!("{COMPOSER_API_URL}/projects/{project_id}/locations/-/environments");
            if let Some(token) = &page_token {
                url.push_str(&format!("?pageToken={token}"));
            }

            debug!("Fetching environments from: {url}");

            let response = self
                .http_client
                .get(&url)
                .header(AUTHORIZATION, format!("Bearer {access_token}"))
                .send()
                .await
                .context(format!(
                    "Failed to list Composer environments for project {project_id}"
                ))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                anyhow::bail!(
                    "Failed to list environments for {project_id}: HTTP {status} - {body}"
                );
            }

            let environments_response: EnvironmentsResponse = response
                .json()
                .await
                .context("Failed to parse environments response")?;

            if let Some(environments) = environments_response.environments {
                all_environments.extend(environments);
            }

            match environments_response.next_page_token {
                Some(token) if !token.is_empty() => page_token = Some(token),
                _ => break,
            }
        }

        Ok(all_environments)
    }
}
```

**Step 3: Verify it compiles**

Run: `cargo check`
Expected: Compiles successfully

---

### Task 6: Implement Environment Discovery Function

**Files:**
- Modify: `src/airflow/managed_services/composer.rs`

**Step 1: Add helper function to parse Airflow version**

Append to `src/airflow/managed_services/composer.rs`:

```rust

use crate::airflow::config::{AirflowAuth, AirflowConfig, AirflowVersion, ManagedService};

/// Parses Airflow version from Composer image version string
/// Format: "composer-X.Y.Z-airflow-A.B.C"
fn parse_airflow_version(image_version: &str) -> AirflowVersion {
    if let Some(airflow_part) = image_version.split("-airflow-").nth(1) {
        if airflow_part.starts_with("3.") {
            return AirflowVersion::V3;
        }
    }
    AirflowVersion::V2
}

/// Extracts the environment name from the full resource name
/// Format: "projects/{project}/locations/{location}/environments/{name}"
fn extract_environment_name(full_name: &str) -> Option<&str> {
    full_name.rsplit('/').next()
}
```

**Step 2: Add the main discovery function**

Append to `src/airflow/managed_services/composer.rs`:

```rust

/// Lists all Composer environments across all accessible projects and returns them as AirflowConfig instances
pub async fn get_composer_environment_servers() -> Result<Vec<AirflowConfig>> {
    let client = ComposerClient::new()?;
    let access_token = ComposerClient::get_access_token().await?;

    let projects = client.list_projects(&access_token).await?;
    let mut servers = Vec::new();

    for project in projects {
        let environments = match client
            .list_environments(&project.project_id, &access_token)
            .await
        {
            Ok(envs) => envs,
            Err(e) => {
                error!(
                    "Failed to list environments for project '{}': {}",
                    project.project_id, e
                );
                continue;
            }
        };

        for env in environments {
            // Only include running environments
            if env.state.as_deref() != Some("RUNNING") {
                debug!(
                    "Skipping environment {} with state {:?}",
                    env.name, env.state
                );
                continue;
            }

            let config = match &env.config {
                Some(c) => c,
                None => {
                    error!("Environment {} has no config", env.name);
                    continue;
                }
            };

            let airflow_uri = match &config.airflow_uri {
                Some(uri) => uri.clone(),
                None => {
                    error!("Environment {} has no airflow_uri", env.name);
                    continue;
                }
            };

            let image_version = config
                .software_config
                .as_ref()
                .and_then(|sc| sc.image_version.as_ref())
                .map(String::as_str)
                .unwrap_or("unknown");

            let version = parse_airflow_version(image_version);

            let env_name = extract_environment_name(&env.name).unwrap_or("unknown");

            // Ensure endpoint has trailing slash
            let endpoint = if airflow_uri.ends_with('/') {
                airflow_uri
            } else {
                format!("{airflow_uri}/")
            };

            info!(
                "Discovered Composer environment: {}/{} ({})",
                project.project_id, env_name, endpoint
            );

            servers.push(AirflowConfig {
                name: format!("{}/{}", project.project_id, env_name),
                endpoint,
                auth: AirflowAuth::Composer(ComposerAuth {
                    project_id: project.project_id.clone(),
                    environment_name: env_name.to_string(),
                    access_token: access_token.clone(),
                }),
                managed: Some(ManagedService::Gcc),
                version,
                timeout_secs: 30,
            });
        }
    }

    info!("Found {} Composer environment(s)", servers.len());
    Ok(servers)
}
```

**Step 3: Verify it compiles**

Run: `cargo check`
Expected: Compiles successfully

---

### Task 7: Integrate with Config Expansion

**Files:**
- Modify: `src/airflow/config/mod.rs:13-16, 190-192`

**Step 1: Add import for composer function**

At line 15, add the import:

```rust
use super::managed_services::composer::get_composer_environment_servers;
```

**Step 2: Replace the Gcc placeholder with actual implementation**

Replace lines 190-192:

```rust
                ManagedService::Gcc => {
                    log::warn!("ManagedService::Gcc (Google Cloud Composer) expansion not implemented; skipping");
                }
```

With:

```rust
                ManagedService::Gcc => {
                    match get_composer_environment_servers().await {
                        Ok(composer_servers) => {
                            self.extend_servers(composer_servers);
                        }
                        Err(e) => {
                            log::error!("Failed to get Composer environments: {e}");
                        }
                    }
                }
```

**Step 3: Verify it compiles**

Run: `cargo check`
Expected: Compiles successfully

---

### Task 8: Run Tests and Fix Any Issues

**Step 1: Run all tests**

Run: `cargo test`
Expected: All tests pass

**Step 2: Run clippy**

Run: `cargo clippy`
Expected: No errors (warnings are acceptable)

**Step 3: Build release**

Run: `cargo build --release`
Expected: Builds successfully

---

### Task 9: Add Unit Tests for Version Parsing

**Files:**
- Modify: `src/airflow/managed_services/composer.rs`

**Step 1: Add test module**

Append to `src/airflow/managed_services/composer.rs`:

```rust

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_airflow_version_v2() {
        assert_eq!(
            parse_airflow_version("composer-2.9.7-airflow-2.9.3"),
            AirflowVersion::V2
        );
        assert_eq!(
            parse_airflow_version("composer-2.0.0-airflow-2.0.0"),
            AirflowVersion::V2
        );
    }

    #[test]
    fn test_parse_airflow_version_v3() {
        assert_eq!(
            parse_airflow_version("composer-3.0.0-airflow-3.0.0"),
            AirflowVersion::V3
        );
        assert_eq!(
            parse_airflow_version("composer-3.1.0-airflow-3.1.0"),
            AirflowVersion::V3
        );
    }

    #[test]
    fn test_parse_airflow_version_unknown() {
        // Unknown formats default to V2
        assert_eq!(parse_airflow_version("unknown"), AirflowVersion::V2);
        assert_eq!(parse_airflow_version(""), AirflowVersion::V2);
    }

    #[test]
    fn test_extract_environment_name() {
        assert_eq!(
            extract_environment_name("projects/my-project/locations/us-central1/environments/prod"),
            Some("prod")
        );
        assert_eq!(
            extract_environment_name("projects/p/locations/l/environments/my-env"),
            Some("my-env")
        );
        assert_eq!(extract_environment_name("simple-name"), Some("simple-name"));
    }
}
```

**Step 2: Run the tests**

Run: `cargo test composer`
Expected: All tests pass

---

### Task 10: Final Verification

**Step 1: Run full test suite**

Run: `cargo test`
Expected: All tests pass

**Step 2: Run clippy with all warnings**

Run: `cargo clippy -- -W clippy::all`
Expected: No errors

**Step 3: Verify the application runs**

Run: `FLOWRS_LOG=debug cargo run -- --help`
Expected: Shows help output without errors

---

## Summary

After completing all tasks, the following will be implemented:

1. **New dependency:** `google-cloud-auth` for GCP authentication
2. **New module:** `src/airflow/managed_services/composer.rs` with:
   - `ComposerAuth` struct for storing credentials
   - `ComposerClient` for API calls
   - `get_composer_environment_servers()` for auto-discovery
3. **Modified files:**
   - `Cargo.toml` - added dependency
   - `src/airflow/managed_services.rs` - exported composer module
   - `src/airflow/config/mod.rs` - added Composer auth variant and expansion
   - `src/airflow/client/base.rs` - added Composer auth handling

To use: Add `managed_services = ["Gcc"]` to config.toml
