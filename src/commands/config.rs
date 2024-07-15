use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use inquire::validator::Validation;
use url::Url;

use crate::app::config::FlowrsConfig;
use crate::app::error::Result;
use crate::CONFIG_FILE;

pub mod add;
pub mod model;
pub mod remove;
pub mod update;

fn validate_endpoint(
    endpoint: &str,
) -> std::result::Result<Validation, Box<dyn std::error::Error + Send + Sync>> {
    match Url::parse(endpoint) {
        Ok(_) => Ok(Validation::Valid),
        Err(error) => Ok(Validation::Invalid(error.into())),
    }
}

fn write_config(config: &FlowrsConfig, path: Option<&Path>) -> Result<()> {
    let path = match path {
        Some(path) => path,
        None => CONFIG_FILE.as_path(),
    };
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)?;

    file.write_all(toml::to_string(config)?.as_bytes())?;
    Ok(())
}
