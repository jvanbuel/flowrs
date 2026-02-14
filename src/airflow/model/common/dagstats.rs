use crate::airflow::client::v1;
use crate::airflow::client::v2;
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

// From trait implementations for v1 models
impl From<v1::model::dagstats::DagStatsResponse> for DagStatsResponse {
    fn from(value: v1::model::dagstats::DagStatsResponse) -> Self {
        Self {
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
        Self {
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
        Self {
            state: DagRunState::from(value.state.as_str()),
            count: value.count,
        }
    }
}

// From trait implementations for v2 models
impl From<v2::model::dagstats::DagStatsResponse> for DagStatsResponse {
    fn from(value: v2::model::dagstats::DagStatsResponse) -> Self {
        Self {
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
        Self {
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
        Self {
            state: DagRunState::from(value.state.as_str()),
            count: value.count,
        }
    }
}
