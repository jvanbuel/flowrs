use anyhow::Result;
use log::info;

use crate::config::{AirflowConfig, ManagedService};

#[cfg(feature = "astronomer")]
use super::astronomer::get_astronomer_environment_servers;
#[cfg(feature = "composer")]
use super::composer::{get_composer_environment_servers, get_gcloud_default_region};
#[cfg(feature = "conveyor")]
use super::conveyor::get_conveyor_environment_servers;
#[cfg(feature = "mwaa")]
use super::mwaa::get_mwaa_environment_servers;

/// Configuration for managed service expansion, decomposed from FlowrsConfig
/// so that flowrs-airflow does not depend on the config crate.
pub struct ManagedServiceConfig {
    pub services: Vec<ManagedService>,
    pub gcc_regions: Vec<String>,
    pub gcc_projects: Option<Vec<String>>,
}

/// Expands managed services by resolving their environments and returning server configs.
/// Returns a tuple of (new servers, errors) where errors contains any non-fatal errors encountered.
pub async fn expand_managed_services(
    #[allow(unused_variables)] config: ManagedServiceConfig,
) -> Result<(Vec<AirflowConfig>, Vec<String>)> {
    let mut all_servers = Vec::new();
    let mut all_errors = Vec::new();

    if config.services.is_empty() {
        return Ok((all_servers, all_errors));
    }

    for service in &config.services {
        match service {
            ManagedService::Conveyor => {
                #[cfg(feature = "conveyor")]
                {
                    let conveyor_servers = get_conveyor_environment_servers()?;
                    all_servers.extend(conveyor_servers);
                }
                #[cfg(not(feature = "conveyor"))]
                {
                    all_errors.push(
                        "Conveyor support not compiled. Enable the 'conveyor' feature.".to_string(),
                    );
                }
            }
            ManagedService::Mwaa => {
                #[cfg(feature = "mwaa")]
                {
                    let mwaa_servers = get_mwaa_environment_servers().await?;
                    all_servers.extend(mwaa_servers);
                }
                #[cfg(not(feature = "mwaa"))]
                {
                    all_errors
                        .push("MWAA support not compiled. Enable the 'mwaa' feature.".to_string());
                }
            }
            ManagedService::Astronomer => {
                #[cfg(feature = "astronomer")]
                {
                    let (astronomer_servers, errors) = get_astronomer_environment_servers().await;
                    all_errors.extend(errors);
                    all_servers.extend(astronomer_servers);
                }
                #[cfg(not(feature = "astronomer"))]
                {
                    all_errors.push(
                        "Astronomer support not compiled. Enable the 'astronomer' feature."
                            .to_string(),
                    );
                }
            }
            ManagedService::Gcc => {
                #[cfg(feature = "composer")]
                {
                    let configured_regions = config.gcc_regions.clone();

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

                    let project_ids = config.gcc_projects.as_ref().filter(|p| !p.is_empty());

                    match get_composer_environment_servers(&regions, project_ids.map(Vec::as_slice))
                        .await
                    {
                        Ok(composer_servers) => {
                            all_servers.extend(composer_servers);
                        }
                        Err(e) => {
                            log::error!("Failed to get Composer environments: {e}");
                            all_errors.push(format!("Google Cloud Composer: {e}"));
                        }
                    }
                }
                #[cfg(not(feature = "composer"))]
                {
                    all_errors.push(
                        "Google Cloud Composer support not compiled. Enable the 'composer' feature."
                            .to_string(),
                    );
                }
            }
        }
    }
    info!(
        "Expanded managed services: servers={}, errors={}",
        all_servers.len(),
        all_errors.len()
    );
    Ok((all_servers, all_errors))
}
