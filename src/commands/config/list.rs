use std::path::Path;

use super::model::ListCommand;
use crate::airflow::config::FlowrsConfig;
use crate::app::error::Result;

impl ListCommand {
    pub async fn run(&self) -> Result<()> {
        let path = self.file.as_ref().map(Path::new);
        let config = FlowrsConfig::from_file(path).await?;
        let servers = config.servers.unwrap_or_default();

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
