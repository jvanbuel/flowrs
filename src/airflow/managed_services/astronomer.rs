use crate::airflow::config::{AirflowAuth, AirflowConfig, AirflowVersion, ManagedService};
use anyhow::{Context, Result};
use log::info;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt;
use std::sync::LazyLock;
use std::time::Duration;

static FLOWRS_USER_AGENT: LazyLock<String> = LazyLock::new(|| {
    let version = env!("CARGO_PKG_VERSION");
    format!("flowrs/{version}")
});

/// Astronomer client for managing authentication and deployment discovery
#[derive(Debug, Clone)]
pub struct AstronomerClient {
    client: reqwest::Client,
    api_token: String,
    base_url: String,
}

impl AstronomerClient {
    /// Creates a new Astronomer client using the `ASTRO_API_TOKEN` environment variable
    pub fn new() -> Result<Self> {
        let api_token =
            env::var("ASTRO_API_TOKEN").context("ASTRO_API_TOKEN environment variable not set")?;

        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(&FLOWRS_USER_AGENT));

        let client = reqwest::Client::builder()
            .use_rustls_tls()
            .timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(3))
            .default_headers(headers)
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            client,
            api_token,
            base_url: "https://api.astronomer.io/platform/v1beta1".to_string(),
        })
    }

    /// Lists all organizations
    pub async fn list_organizations(&self) -> Result<Vec<Organization>> {
        const PAGE_SIZE: u32 = 100;
        let mut all_organizations = Vec::new();
        let mut offset = 0;

        loop {
            let url = format!(
                "{}/organizations?offset={}&limit={}",
                self.base_url, offset, PAGE_SIZE
            );

            let response = self
                .client
                .get(&url)
                .bearer_auth(&self.api_token)
                .send()
                .await
                .context("Failed to list Astronomer organizations")?;

            if !response.status().is_success() {
                anyhow::bail!("Failed to list organizations: HTTP {}", response.status());
            }

            let org_response: OrganizationsResponse = response
                .json()
                .await
                .context("Failed to parse organizations response")?;

            let items_count = org_response.organizations.len();
            all_organizations.extend(org_response.organizations);

            // Guard against infinite loops: break if no items or limit is zero
            if items_count == 0 || org_response.limit == 0 {
                break;
            }

            offset += org_response.limit;

            // Stop if we've fetched all items
            if offset >= org_response.total_count {
                break;
            }
        }

        Ok(all_organizations)
    }

    /// Lists all deployments for a specific organization
    pub async fn list_deployments(&self, organization_id: &str) -> Result<Vec<Deployment>> {
        const PAGE_SIZE: u32 = 100;
        let mut all_deployments = Vec::new();
        let mut offset = 0;

        loop {
            let url = format!(
                "{}/organizations/{}/deployments?offset={}&limit={}",
                self.base_url, organization_id, offset, PAGE_SIZE
            );

            let response = self
                .client
                .get(&url)
                .bearer_auth(&self.api_token)
                .send()
                .await
                .context(format!(
                    "Failed to list deployments for organization {organization_id}"
                ))?;

            if !response.status().is_success() {
                anyhow::bail!(
                    "Failed to list deployments for organization {}: HTTP {}",
                    organization_id,
                    response.status()
                );
            }

            let deployment_response: DeploymentsResponse = response
                .json()
                .await
                .context("Failed to parse deployments response")?;

            let items_count = deployment_response.deployments.len();
            all_deployments.extend(deployment_response.deployments);

            // Guard against infinite loops: break if no items or limit is zero
            if items_count == 0 || deployment_response.limit == 0 {
                break;
            }

            offset += deployment_response.limit;

            // Stop if we've fetched all items
            if offset >= deployment_response.total_count {
                break;
            }
        }

        Ok(all_deployments)
    }
}

/// Astronomer organization metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct OrganizationsResponse {
    organizations: Vec<Organization>,
    total_count: u32,
    offset: u32,
    limit: u32,
}

/// Astronomer deployment metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Deployment {
    pub id: String,
    pub name: String,
    pub organization_id: String,
    pub airflow_version: String,
    pub web_server_url: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeploymentsResponse {
    deployments: Vec<Deployment>,
    total_count: u32,
    offset: u32,
    limit: u32,
}

/// Astronomer authentication data including API token
#[derive(Clone, Serialize, Deserialize)]
pub struct AstronomerAuth {
    pub api_token: String,
}

impl fmt::Debug for AstronomerAuth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AstronomerAuth")
            .field("api_token", &"***redacted***")
            .finish()
    }
}

/// Lists all Astronomer deployments across all organizations and returns them as `AirflowConfig` instances
/// Returns a tuple of (successful configs, error messages for failed organizations)
pub async fn get_astronomer_environment_servers() -> (Vec<AirflowConfig>, Vec<String>) {
    let mut servers = Vec::new();
    let mut errors = Vec::new();

    let client = match AstronomerClient::new() {
        Ok(client) => client,
        Err(e) => {
            errors.push(format!("Failed to create Astronomer client: {e}"));
            return (servers, errors);
        }
    };

    let organizations = match client.list_organizations().await {
        Ok(orgs) => orgs,
        Err(e) => {
            errors.push(format!("Failed to list organizations: {e}"));
            return (servers, errors);
        }
    };

    info!("Found {} Astronomer organization(s)", organizations.len());

    for org in organizations {
        // Skip inactive organizations
        if org.status != "ACTIVE" {
            continue;
        }

        let deployments = match client.list_deployments(&org.id).await {
            Ok(deployments) => deployments,
            Err(e) => {
                errors.push(format!(
                    "Failed to list deployments for organization '{}': {}",
                    org.name, e
                ));
                continue; // Continue with next organization even if this one fails
            }
        };

        for deployment in deployments {
            // Determine Airflow version from the version string
            let version = if deployment.airflow_version.starts_with("2.") {
                AirflowVersion::V2
            } else if deployment.airflow_version.starts_with("3.") {
                AirflowVersion::V3
            } else {
                errors.push(format!(
                    "Unsupported Airflow version '{}' for deployment '{}' in organization '{}'",
                    deployment.airflow_version, deployment.name, org.name
                ));
                continue;
            };

            // Ensure the endpoint has a proper scheme and trailing slash
            let mut endpoint = if deployment.web_server_url.starts_with("http://")
                || deployment.web_server_url.starts_with("https://")
            {
                deployment.web_server_url.clone()
            } else {
                format!("https://{}", deployment.web_server_url)
            };

            // Add trailing slash if not present (required for correct URL joining)
            if !endpoint.ends_with('/') {
                endpoint.push('/');
            }

            info!(
                "Discovered Astronomer deployment: {}/{} ({})",
                org.name, deployment.name, endpoint
            );

            servers.push(AirflowConfig {
                name: format!("{}/{}", org.name, deployment.name),
                endpoint,
                auth: AirflowAuth::Astronomer(AstronomerAuth {
                    api_token: client.api_token.clone(),
                }),
                managed: Some(ManagedService::Astronomer),
                version,
                timeout_secs: 30,
            });
        }
    }

    info!(
        "Found {} Astronomer deployment(s) with {} error(s)",
        servers.len(),
        errors.len()
    );
    (servers, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_astronomer_environments() {
        let (environments, errors) = get_astronomer_environment_servers().await;
        // This test will only work if you have ASTRO_API_TOKEN configured
        // and Astronomer deployments available
        println!("Found {} Astronomer deployments", environments.len());
        for env in environments {
            println!("  - {} ({})", env.name, env.endpoint);
        }
        if !errors.is_empty() {
            println!("Errors: {errors:?}");
        }
    }

    #[tokio::test]
    async fn test_astronomer_client_new() {
        let client = AstronomerClient::new();
        if let Err(e) = &client {
            println!("Expected error if ASTRO_API_TOKEN not set: {e}");
        }
    }
}
