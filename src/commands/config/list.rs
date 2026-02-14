use std::path::PathBuf;

use super::model::ListCommand;
use crate::airflow::config::FlowrsConfig;
use anyhow::Result;

impl ListCommand {
    pub fn run(&self) -> Result<()> {
        let path = self.file.as_ref().map(PathBuf::from);
        let config = FlowrsConfig::from_file(path.as_ref())?;
        let servers = config.servers;

        if servers.is_empty() {
            println!("❌ No servers found in the config file!");
        } else {
            println!("📋 Airflow instances in the config file:");
            for server in servers {
                if let Some(managed) = server.managed {
                    println!("  - {} ({})", server.name, managed);
                } else {
                    println!("  - {}", server.name);
                }
            }
        }
        Ok(())
    }
}
