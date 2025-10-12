use std::path::PathBuf;

use inquire::Select;

use super::model::RemoveCommand;
use crate::airflow::config::FlowrsConfig;
use anyhow::Result;

impl RemoveCommand {
    pub fn run(&self) -> Result<()> {
        let path = self.file.as_ref().map(PathBuf::from);
        let mut config = FlowrsConfig::from_file(path.as_ref())?;

        if let Some(mut servers) = config.servers.clone() {
            let name = match self.name {
                None => Select::new(
                    "name",
                    servers.iter().map(|server| server.name.clone()).collect(),
                )
                .prompt()?,
                Some(ref name) => name.clone(),
            };
            servers.retain(|server| server.name != name && server.managed.is_none());
            config.servers = Some(servers);
            config.write_to_file()?;

            println!("✅ Config '{name}' removed successfully!");
        }
        Ok(())
    }
}
