use anyhow::{Context, Result};
use futures::future::join_all;
use google_cloud_auth::credentials::Builder;
use log::{debug, error, info};
use reqwest::header::AUTHORIZATION;
use serde::Deserialize;
use std::time::Duration;

use crate::airflow::config::{AirflowAuth, AirflowConfig, AirflowVersion, ManagedService};

use super::auth::ComposerAuth;

const RESOURCE_MANAGER_URL: &str = "https://cloudresourcemanager.googleapis.com/v1/projects";
const COMPOSER_API_URL: &str = "https://composer.googleapis.com/v1";

/// GCP Project from Resource Manager API
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub project_id: String,
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
            let mut request = self
                .http_client
                .get(RESOURCE_MANAGER_URL)
                .query(&[("filter", "lifecycleState:ACTIVE")])
                .header(AUTHORIZATION, format!("Bearer {access_token}"));

            if let Some(token) = &page_token {
                request = request.query(&[("pageToken", token.as_str())]);
            }

            debug!("Fetching projects");

            let response = request
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

    /// Lists Composer environments in a specific region
    async fn list_environments_in_region(
        &self,
        project_id: &str,
        region: &str,
        access_token: &str,
    ) -> Result<Vec<ComposerEnvironment>> {
        let mut all_environments = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let url =
                format!("{COMPOSER_API_URL}/projects/{project_id}/locations/{region}/environments");

            let mut request = self
                .http_client
                .get(&url)
                .header(AUTHORIZATION, format!("Bearer {access_token}"));

            if let Some(token) = &page_token {
                request = request.query(&[("pageToken", token.as_str())]);
            }

            debug!("Fetching environments for {project_id}/{region}");

            let response = request.send().await.context(format!(
                "Failed to list Composer environments for {project_id}/{region}"
            ))?;

            if !response.status().is_success() {
                // Composer API may not be enabled in all regions — skip silently
                debug!(
                    "Skipping region {region} for {project_id}: HTTP {}",
                    response.status()
                );
                return Ok(Vec::new());
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

    /// Lists all Composer environments in a project across the specified regions
    pub async fn list_environments(
        &self,
        project_id: &str,
        regions: &[impl AsRef<str>],
        access_token: &str,
    ) -> Result<Vec<ComposerEnvironment>> {
        let futures: Vec<_> = regions
            .iter()
            .map(|region| {
                self.list_environments_in_region(project_id, region.as_ref(), access_token)
            })
            .collect();

        let results = join_all(futures).await;

        let mut all_environments = Vec::new();
        for result in results {
            match result {
                Ok(envs) => all_environments.extend(envs),
                Err(e) => debug!("Error listing environments: {e}"),
            }
        }

        Ok(all_environments)
    }
}

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

/// Lists Composer environments and returns them as `AirflowConfig` instances.
/// If `project_ids` is `Some`, only those projects are searched. Otherwise, all accessible projects are searched.
pub async fn get_composer_environment_servers(
    regions: &[impl AsRef<str>],
    project_ids: Option<&[String]>,
) -> Result<Vec<AirflowConfig>> {
    let client = ComposerClient::new()?;
    let access_token = ComposerClient::get_access_token().await?;

    let projects = match project_ids {
        Some(ids) => ids
            .iter()
            .map(|id| Project {
                project_id: id.clone(),
            })
            .collect(),
        None => client.list_projects(&access_token).await?,
    };
    let mut servers = Vec::new();

    for project in projects {
        let environments = match client
            .list_environments(&project.project_id, regions, &access_token)
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

            let Some(config) = &env.config else {
                error!("Environment {} has no config", env.name);
                continue;
            };

            let Some(airflow_uri) = &config.airflow_uri else {
                error!("Environment {} has no airflow_uri", env.name);
                continue;
            };
            let airflow_uri = airflow_uri.clone();

            let image_version = config
                .software_config
                .as_ref()
                .and_then(|sc| sc.image_version.as_ref())
                .map_or("unknown", String::as_str);

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
