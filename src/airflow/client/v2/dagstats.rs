use anyhow::Result;
use async_trait::async_trait;
use reqwest::Method;

use super::model;
use crate::airflow::model::common::dagrun::DagRunState;
use crate::airflow::model::common::dagstats::{DagStatistic, DagStatistics};
use crate::airflow::model::common::DagStatsResponse;
use crate::airflow::traits::DagStatsOperations;

use super::V2Client;

#[async_trait]
impl DagStatsOperations for V2Client {
    async fn get_dag_stats(&self, dag_ids: Vec<&str>) -> Result<DagStatsResponse> {
        let response = self
            .base_api(Method::GET, "dagStats")
            .await?
            .query(
                &dag_ids
                    .into_iter()
                    .map(|id| ("dag_ids", id))
                    .collect::<Vec<_>>(),
            )
            .send()
            .await?
            .error_for_status()?;
        let dag_stats = response.json::<model::dagstats::DagStatsResponse>().await?;
        Ok(dag_stats.into())
    }
}

// From trait implementations for v2 models
impl From<model::dagstats::DagStatsResponse> for DagStatsResponse {
    fn from(value: model::dagstats::DagStatsResponse) -> Self {
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

impl From<model::dagstats::DagStatistics> for DagStatistics {
    fn from(value: model::dagstats::DagStatistics) -> Self {
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

impl From<model::dagstats::DagStatistic> for DagStatistic {
    fn from(value: model::dagstats::DagStatistic) -> Self {
        Self {
            state: DagRunState::from(value.state.as_str()),
            count: value.count,
        }
    }
}
