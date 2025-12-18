// Library exports for integration tests and external usage
//
// Clippy pedantic allows - these are library documentation concerns, not relevant for a TUI application
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]

use std::sync::LazyLock;

pub mod airflow;
pub mod app;
pub mod commands;
pub mod ui;

use airflow::config::paths::ConfigPaths;

pub static CONFIG_PATHS: LazyLock<ConfigPaths> = LazyLock::new(ConfigPaths::resolve);
