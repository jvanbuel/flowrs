// Library exports for integration tests and external usage
//
// Clippy pedantic allows - these are library documentation concerns, not relevant for a TUI application
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]
// The option_if_let_else lint often reduces readability by forcing map_or_else
#![allow(clippy::option_if_let_else)]
// Drop timing hints are often too aggressive for typical application code
#![allow(clippy::significant_drop_tightening)]
// The useless_let_if_seq lint often reduces readability
#![allow(clippy::useless_let_if_seq)]

use std::sync::LazyLock;

pub mod airflow;
pub mod app;
pub mod commands;
pub mod ui;

use airflow::config::paths::ConfigPaths;

pub static CONFIG_PATHS: LazyLock<ConfigPaths> = LazyLock::new(ConfigPaths::resolve);
