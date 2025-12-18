use crate::airflow::client::v1;
use crate::airflow::client::v2;
use serde::{Deserialize, Serialize};

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
    pub state: String,
    pub count: u64,
}

// From trait implementations for v1 models
impl From<v1::model::dagstats::DagStatsResponse> for DagStatsResponse {
    fn from(value: v1::model::dagstats::DagStatsResponse) -> Self {
        DagStatsResponse {
            dags: value
                .dags
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
            total_entries: value.total_entries,
        }
    }
}

impl From<v1::model::dagstats::DagStatistics> for DagStatistics {
    fn from(value: v1::model::dagstats::DagStatistics) -> Self {
        DagStatistics {
            dag_id: value.dag_id,
            stats: value
                .stats
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
        }
    }
}

impl From<v1::model::dagstats::DagStatistic> for DagStatistic {
    fn from(value: v1::model::dagstats::DagStatistic) -> Self {
        DagStatistic {
            state: value.state,
            count: value.count,
        }
    }
}

// From trait implementations for v2 models
impl From<v2::model::dagstats::DagStatsResponse> for DagStatsResponse {
    fn from(value: v2::model::dagstats::DagStatsResponse) -> Self {
        DagStatsResponse {
            dags: value
                .dags
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
            total_entries: value.total_entries,
        }
    }
}

impl From<v2::model::dagstats::DagStatistics> for DagStatistics {
    fn from(value: v2::model::dagstats::DagStatistics) -> Self {
        DagStatistics {
            dag_id: value.dag_id,
            stats: value
                .stats
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
        }
    }
}

impl From<v2::model::dagstats::DagStatistic> for DagStatistic {
    fn from(value: v2::model::dagstats::DagStatistic) -> Self {
        DagStatistic {
            state: value.state,
            count: value.count,
        }
    }
}
