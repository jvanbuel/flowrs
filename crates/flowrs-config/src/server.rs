use std::fmt::{Display, Formatter};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::auth::AirflowAuth;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Default)]
pub enum AirflowVersion {
    #[default]
    V2,
    V3,
}

impl AirflowVersion {
    pub const fn api_path(&self) -> &str {
        match self {
            Self::V2 => "api/v1",
            Self::V3 => "api/v2",
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, ValueEnum, EnumIter)]
pub enum ManagedService {
    Conveyor,
    Mwaa,
    Astronomer,
    Gcc,
}

impl Display for ManagedService {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Conveyor => write!(f, "Conveyor"),
            Self::Mwaa => write!(f, "MWAA"),
            Self::Astronomer => write!(f, "Astronomer"),
            Self::Gcc => write!(f, "Google Cloud Composer"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct GccConfig {
    pub regions: Vec<String>,
    /// GCP project IDs to search for Composer environments.
    /// `None` means search all accessible projects.
    pub projects: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AirflowConfig {
    pub name: String,
    pub endpoint: String,
    pub auth: AirflowAuth,
    pub managed: Option<ManagedService>,
    #[serde(default)]
    pub version: AirflowVersion,
    /// Request timeout in seconds. Defaults to 30 seconds if not specified.
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

pub(crate) const fn default_timeout() -> u64 {
    30
}
