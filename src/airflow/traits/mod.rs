pub mod dag;
pub mod dagrun;
pub mod dagstats;
pub mod log;
pub mod taskinstance;

pub use dag::DagOperations;
pub use dagrun::DagRunOperations;
pub use dagstats::DagStatsOperations;
pub use log::LogOperations;
pub use taskinstance::TaskInstanceOperations;

/// Super-trait combining all Airflow API operations.
/// This trait can be implemented by different API versions (v1 for Airflow v2, v2 for Airflow v3)
/// to provide a consistent interface for interacting with Airflow.
pub trait AirflowClient:
    DagOperations + DagRunOperations + TaskInstanceOperations + LogOperations + DagStatsOperations
{
}

// Blanket implementation: any type that implements all the subtraits automatically implements AirflowClient
impl<T> AirflowClient for T where
    T: DagOperations
        + DagRunOperations
        + TaskInstanceOperations
        + LogOperations
        + DagStatsOperations
{
}
