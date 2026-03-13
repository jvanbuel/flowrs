use anyhow::Result;
use log::info;

use crate::airflow::config::{FlowrsConfig, ManagedService};

use super::astronomer::get_astronomer_environment_servers;
use super::composer::{get_composer_environment_servers, get_gcloud_default_region};
use super::conveyor::get_conveyor_environment_servers;
use super::mwaa::get_mwaa_environment_servers;

/// Expands the config by resolving managed services and adding their servers.
/// This is an async function that should be called after `from_file`/`from_str`
/// when you need to resolve managed service environments.
/// Returns a tuple of (config, errors) where errors contains any non-fatal errors encountered.
pub async fn expand_managed_services(
    mut config: FlowrsConfig,
) -> Result<(FlowrsConfig, Vec<String>)> {
    let mut all_errors = Vec::new();

    if config.managed_services.is_empty() {
        return Ok((config, all_errors));
    }

    let services = config.managed_services.clone();
    for service in services {
        match service {
            ManagedService::Conveyor => {
                let conveyor_servers = get_conveyor_environment_servers()?;
                config.extend_servers(conveyor_servers);
            }
            ManagedService::Mwaa => {
                let mwaa_servers = get_mwaa_environment_servers().await?;
                config.extend_servers(mwaa_servers);
            }
            ManagedService::Astronomer => {
                let (astronomer_servers, errors) = get_astronomer_environment_servers().await;
                all_errors.extend(errors);
                config.extend_servers(astronomer_servers);
            }
            ManagedService::Gcc => {
                let configured_regions: Vec<String> = config
                    .gcc
                    .as_ref()
                    .map(|c| c.regions.clone())
                    .unwrap_or_default();

                let regions = if configured_regions.is_empty() {
                    let default_region = tokio::task::spawn_blocking(get_gcloud_default_region)
                        .await
                        .ok()
                        .flatten();
                    if let Some(region) = default_region {
                        info!("No [gcc] regions configured, using gcloud default: {region}");
                        vec![region]
                    } else {
                        all_errors.push(
                            "Google Cloud Composer: no regions configured.\n\
                             Add a [gcc] section to your config:\n\n\
                             [gcc]\n\
                             regions = [\"europe-west1\"]\n\n\
                             Or set a default region: gcloud config set compute/region <region>"
                                .to_string(),
                        );
                        continue;
                    }
                } else {
                    configured_regions
                };

                let project_ids = config
                    .gcc
                    .as_ref()
                    .and_then(|c| c.projects.as_ref())
                    .filter(|p| !p.is_empty());

                match get_composer_environment_servers(&regions, project_ids.map(Vec::as_slice))
                    .await
                {
                    Ok(composer_servers) => {
                        config.extend_servers(composer_servers);
                    }
                    Err(e) => {
                        log::error!("Failed to get Composer environments: {e}");
                        all_errors.push(format!("Google Cloud Composer: {e}"));
                    }
                }
            }
        }
    }
    info!(
        "Expanded config: servers={}, errors={}",
        config.servers.len(),
        all_errors.len()
    );
    Ok((config, all_errors))
}
