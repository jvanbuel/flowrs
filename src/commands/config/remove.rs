use std::path::Path;

use inquire::Select;

use super::model::RemoveCommand;
use crate::{
    app::{config::FlowrsConfig, error::Result},
    commands::config::write_config,
};

impl RemoveCommand {
    pub fn run(&self) -> Result<()> {
        let path = self.file.as_ref().map(Path::new);
        let config = FlowrsConfig::from_file(path)?;

        if let Some(mut servers) = config.servers.clone() {
            let name = match self.name {
                None => Select::new(
                    "name",
                    servers.iter().map(|server| server.name.clone()).collect(),
                )
                .prompt()?,
                Some(ref name) => name.to_string(),
            };
            servers.retain(|server| server.name != name);
            write_config(&config, path)?;

            println!("✅ Config '{}' removed successfully!", name);
        };
        Ok(())
    }
}
