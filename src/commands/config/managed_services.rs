use std::path::PathBuf;

use inquire::{validator::Validation, MultiSelect, Select};
use strum::IntoEnumIterator;

use super::model::ManagedServiceCommand;
use crate::airflow::config::{FlowrsConfig, GccConfig, ManagedService};
use crate::airflow::managed_services::composer::{
    get_gcloud_default_region, ComposerClient, GCP_REGIONS,
};
use anyhow::Result;

impl ManagedServiceCommand {
    pub async fn run(&self) -> Result<()> {
        let managed_service = match &self.managed_service {
            Some(managed_service) => managed_service.clone(),
            None => Select::new("managed service", ManagedService::iter().collect()).prompt()?,
        };

        let path = self.file.as_ref().map(PathBuf::from);
        let mut config = FlowrsConfig::from_file(path.as_ref())?;

        let already_enabled = config.managed_services.contains(&managed_service);
        if already_enabled {
            println!("Managed service already enabled — updating configuration...");
        }

        if managed_service == ManagedService::Gcc {
            let regions = prompt_gcc_regions(config.gcc.as_ref())?;
            let projects = prompt_gcc_projects(config.gcc.as_ref()).await?;
            config.gcc = Some(GccConfig { regions, projects });
        }

        if !already_enabled {
            config.managed_services.push(managed_service);
        }

        config.write_to_file()?;

        if already_enabled {
            println!("✅ Managed service configuration updated successfully!");
        } else {
            println!("✅ Managed service added successfully!");
        }
        Ok(())
    }

    pub fn disable(&self) -> Result<()> {
        let managed_service = match &self.managed_service {
            Some(managed_service) => managed_service.clone(),
            None => Select::new("managed service", ManagedService::iter().collect()).prompt()?,
        };

        let path = self.file.as_ref().map(PathBuf::from);
        let mut config = FlowrsConfig::from_file(path.as_ref())?;

        if !config.managed_services.contains(&managed_service) {
            println!("Managed service already disabled!");
            return Ok(());
        }
        config
            .managed_services
            .retain(|service| service != &managed_service);

        if managed_service == ManagedService::Gcc {
            config.gcc = None;
        }

        config.write_to_file()?;

        println!("✅ Managed service disabled successfully!");
        Ok(())
    }
}

fn prompt_gcc_regions(existing: Option<&GccConfig>) -> Result<Vec<String>> {
    let regions: Vec<String> = GCP_REGIONS.iter().map(|r| (*r).to_string()).collect();

    // Use existing config as defaults, falling back to gcloud default region
    let defaults: Vec<usize> = if let Some(gcc) = existing {
        gcc.regions
            .iter()
            .filter_map(|r| regions.iter().position(|reg| reg == r))
            .collect()
    } else {
        let default_region = get_gcloud_default_region();
        default_region
            .as_ref()
            .and_then(|dr| regions.iter().position(|r| r == dr))
            .into_iter()
            .collect()
    };

    let selected = MultiSelect::new("Select GCP regions for Composer:", regions)
        .with_default(&defaults)
        .with_help_message("↑↓ navigate, Space to select, Enter to confirm")
        .with_validator(|selection: &[inquire::list_option::ListOption<&String>]| {
            if selection.is_empty() {
                Ok(Validation::Invalid(
                    "At least one region must be selected".into(),
                ))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt()?;

    Ok(selected)
}

const ALL_PROJECTS: &str = "All projects";

async fn prompt_gcc_projects(existing: Option<&GccConfig>) -> Result<Option<Vec<String>>> {
    println!("Fetching GCP projects...");

    let client = ComposerClient::new()?;
    let access_token = ComposerClient::get_access_token().await?;
    let projects = client.list_projects(&access_token).await?;

    if projects.is_empty() {
        println!("No GCP projects found.");
        return Ok(None);
    }

    let mut options: Vec<String> = vec![ALL_PROJECTS.to_string()];
    options.extend(projects.iter().map(|p| p.project_id.clone()));

    let configured_projects = existing.and_then(|c| c.projects.as_ref());
    let defaults: Vec<usize> = match configured_projects {
        Some(selected) => options
            .iter()
            .enumerate()
            .filter_map(|(i, opt)| selected.contains(opt).then_some(i))
            .collect(),
        None => vec![0], // "All projects" selected by default
    };

    let selected = MultiSelect::new(
        "Select GCP projects to search for Composer environments:",
        options,
    )
    .with_default(&defaults)
    .with_help_message("↑↓ navigate, Space to select, Enter to confirm")
    .prompt()?;

    if selected.is_empty() || selected.iter().any(|s| s == ALL_PROJECTS) {
        Ok(None)
    } else {
        Ok(Some(selected))
    }
}
