// Library exports for integration tests and external usage

use std::sync::LazyLock;

pub mod airflow;
pub mod app;
pub mod commands;
pub mod ui;

use flowrs_config::paths::ConfigPaths;

pub static CONFIG_PATHS: LazyLock<ConfigPaths> = LazyLock::new(ConfigPaths::resolve);
