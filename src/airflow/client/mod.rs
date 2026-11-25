mod convert_v1;
mod convert_v2;
mod impls;
mod open_url;

use anyhow::Result;

use flowrs_airflow::client::{BaseClient, V1Client, V2Client};
use flowrs_airflow::{AirflowConfig, AirflowVersion};

use crate::airflow::model::common::OpenItem;

use open_url::{build_v1_open_url, build_v2_open_url};

/// Wrapper enum that owns a versioned Airflow HTTP client and exposes the TUI-facing operations.
#[derive(Debug)]
pub enum FlowrsClient {
    V1(V1Client),
    V2(V2Client),
}

impl FlowrsClient {
    /// Create a new `FlowrsClient` from an `AirflowConfig`.
    pub fn new(config: &AirflowConfig) -> Result<Self> {
        let base = BaseClient::new(config.clone())?;
        match config.version {
            AirflowVersion::V2 => Ok(Self::V1(V1Client::new(base))),
            AirflowVersion::V3 => Ok(Self::V2(V2Client::new(base))),
        }
    }
}

impl FlowrsClient {
    /// Whether clearing a mapped task instance targets only its own map index.
    ///
    /// The `/api/v1` clear body only takes plain task ids, so Airflow 2 clears
    /// every map index of the task.
    pub const fn clears_single_map_index(&self) -> bool {
        matches!(self, Self::V2(_))
    }

    pub fn build_open_url(&self, item: &OpenItem) -> Result<String> {
        match self {
            Self::V1(client) => build_v1_open_url(client.endpoint(), item),
            Self::V2(client) => build_v2_open_url(client.endpoint(), item),
        }
    }
}
