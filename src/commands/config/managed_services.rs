use std::path::PathBuf;

use inquire::Select;
use strum::IntoEnumIterator;

use super::model::ManagedServiceCommand;
use crate::airflow::config::FlowrsConfig;
use crate::airflow::config::ManagedService;
use anyhow::Result;

impl ManagedServiceCommand {
    pub fn run(&self) -> Result<()> {
        let managed_service = if let Some(managed_service) = self.managed_service.clone() {
            managed_service
        } else {
            let managed_service: ManagedService =
                Select::new("managed service", ManagedService::iter().collect()).prompt()?;
            managed_service
        };

        let path = self.file.as_ref().map(PathBuf::from);
        let mut config = FlowrsConfig::from_file(path.as_ref())?;

        match config.managed_services {
            Some(ref mut services) => {
                if services.contains(&managed_service) {
                    println!("Managed service already enabled!");
                    return Ok(());
                }
                services.push(managed_service);
            }
            None => {
                config.managed_services = Some(vec![managed_service]);
            }
        }

        config.write_to_file()?;

        println!("✅ Managed service added successfully!");
        Ok(())
    }

    pub fn disable(&self) -> Result<()> {
        let managed_service = if let Some(managed_service) = self.managed_service.clone() {
            managed_service
        } else {
            let managed_service: ManagedService =
                Select::new("managed service", ManagedService::iter().collect()).prompt()?;
            managed_service
        };

        let path = self.file.as_ref().map(PathBuf::from);
        let mut config = FlowrsConfig::from_file(path.as_ref())?;

        if let Some(ref mut services) = config.managed_services {
            if !services.contains(&managed_service) {
                println!("Managed service already disabled!");
                return Ok(());
            }
            services.retain(|service| service != &managed_service);
        }

        config.write_to_file()?;

        println!("✅ Managed service disabled successfully!");
        Ok(())
    }
}
