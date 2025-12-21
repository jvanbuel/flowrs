pub mod dag;
pub mod dagrun;
pub mod dagstats;
pub mod log;
pub mod task;
pub mod taskinstance;

pub use dag::DagOperations;
pub use dagrun::DagRunOperations;
pub use dagstats::DagStatsOperations;
pub use log::LogOperations;
pub use task::TaskOperations;
pub use taskinstance::TaskInstanceOperations;

use crate::airflow::config::AirflowVersion;
use crate::app::worker::OpenItem;
use anyhow::Result;

/// Super-trait combining all Airflow API operations.
/// This trait can be implemented by different API versions (v1 for Airflow v2, v2 for Airflow v3)
/// to provide a consistent interface for interacting with Airflow.
pub trait AirflowClient:
    DagOperations
    + DagRunOperations
    + TaskInstanceOperations
    + LogOperations
    + DagStatsOperations
    + TaskOperations
{
    /// Get the Airflow version this client is configured for
    #[allow(unused)]
    fn get_version(&self) -> AirflowVersion;

    /// Build the appropriate web UI URL for opening an item in the browser.
    /// The URL structure differs between Airflow v2 and v3.
    #[allow(unused)]
    fn build_open_url(&self, item: &OpenItem) -> Result<String>;
}
