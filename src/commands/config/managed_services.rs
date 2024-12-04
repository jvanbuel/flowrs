use std::path::Path;

use inquire::Select;
use strum::IntoEnumIterator;

use super::model::ManagedServiceCommand;
use crate::airflow::config::FlowrsConfig;
use crate::airflow::config::ManagedService;
use anyhow::Result;

impl ManagedServiceCommand {
    pub fn run(&self) -> Result<()> {
        let managed_service = match self.managed_service.clone() {
            Some(managed_service) => managed_service,
            None => {
                let managed_service: ManagedService =
                    Select::new("managed service", ManagedService::iter().collect()).prompt()?;
                managed_service
            }
        };

        let path = self.file.as_ref().map(Path::new);
        let mut config = FlowrsConfig::from_file(path)?;

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

        config.to_file(path)?;

        println!("✅ Managed service added successfully!");
        Ok(())
    }

    pub fn disable(&self) -> Result<()> {
        let managed_service = match self.managed_service.clone() {
            Some(managed_service) => managed_service,
            None => {
                let managed_service: ManagedService =
                    Select::new("managed service", ManagedService::iter().collect()).prompt()?;
                managed_service
            }
        };

        let path = self.file.as_ref().map(Path::new);
        let mut config = FlowrsConfig::from_file(path)?;

        if let Some(ref mut services) = config.managed_services {
            if !services.contains(&managed_service) {
                println!("Managed service already disabled!");
                return Ok(());
            }
            services.retain(|service| service != &managed_service);
        }

        config.to_file(path)?;

        println!("✅ Managed service disabled successfully!");
        Ok(())
    }
}
