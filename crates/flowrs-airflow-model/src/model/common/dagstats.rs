use serde::{Deserialize, Serialize};

use super::dagrun::DagRunState;

/// Common `DagStats` model used by the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagStatsResponse {
    pub dags: Vec<DagStatistics>,
    pub total_entries: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagStatistics {
    pub dag_id: String,
    pub stats: Vec<DagStatistic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagStatistic {
    pub state: DagRunState,
    pub count: u64,
}
