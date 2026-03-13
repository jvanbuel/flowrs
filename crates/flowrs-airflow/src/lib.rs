// Clippy pedantic allows - consistent with the rest of the workspace
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::option_if_let_else)]

pub mod auth;
pub mod client;
pub mod config;
pub mod managed_services;

pub use auth::{AirflowAuth, AstronomerAuth, BasicAuth, ComposerAuth, MwaaAuth, MwaaTokenType, TokenSource};
pub use client::create_client;
pub use config::{AirflowConfig, AirflowVersion, GccConfig, ManagedService};
