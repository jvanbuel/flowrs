use serde::{Deserialize, Serialize};

/// Common DagStats model used by the application
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
    pub state: String,
    pub count: u64,
}

// From trait implementations for v1 models
impl From<crate::airflow::model::v1::dagstats::DagStatsResponse> for DagStatsResponse {
    fn from(value: crate::airflow::model::v1::dagstats::DagStatsResponse) -> Self {
        DagStatsResponse {
            dags: value.dags.into_iter().map(|d| d.into()).collect(),
            total_entries: value.total_entries,
        }
    }
}

impl From<crate::airflow::model::v1::dagstats::DagStatistics> for DagStatistics {
    fn from(value: crate::airflow::model::v1::dagstats::DagStatistics) -> Self {
        DagStatistics {
            dag_id: value.dag_id,
            stats: value.stats.into_iter().map(|s| s.into()).collect(),
        }
    }
}

impl From<crate::airflow::model::v1::dagstats::DagStatistic> for DagStatistic {
    fn from(value: crate::airflow::model::v1::dagstats::DagStatistic) -> Self {
        DagStatistic {
            state: value.state,
            count: value.count,
        }
    }
}

// From trait implementations for v2 models
impl From<crate::airflow::model::v2::dagstats::DagStatsResponse> for DagStatsResponse {
    fn from(value: crate::airflow::model::v2::dagstats::DagStatsResponse) -> Self {
        DagStatsResponse {
            dags: value.dags.into_iter().map(|d| d.into()).collect(),
            total_entries: value.total_entries,
        }
    }
}

impl From<crate::airflow::model::v2::dagstats::DagStatistics> for DagStatistics {
    fn from(value: crate::airflow::model::v2::dagstats::DagStatistics) -> Self {
        DagStatistics {
            dag_id: value.dag_id,
            stats: value.stats.into_iter().map(|s| s.into()).collect(),
        }
    }
}

impl From<crate::airflow::model::v2::dagstats::DagStatistic> for DagStatistic {
    fn from(value: crate::airflow::model::v2::dagstats::DagStatistic) -> Self {
        DagStatistic {
            state: value.state,
            count: value.count,
        }
    }
}
