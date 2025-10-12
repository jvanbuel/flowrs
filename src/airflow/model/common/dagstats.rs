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
    /// Converts a v1 `DagStatsResponse` into the common `DagStatsResponse`.
    ///
    /// # Examples
    ///
    /// ```
    /// let v1 = crate::airflow::model::v1::dagstats::DagStatsResponse {
    ///     dags: vec![],
    ///     total_entries: 0,
    /// };
    /// let common: crate::airflow::model::common::dagstats::DagStatsResponse = v1.into();
    /// assert_eq!(common.total_entries, 0);
    /// ```
    fn from(value: crate::airflow::model::v1::dagstats::DagStatsResponse) -> Self {
        DagStatsResponse {
            dags: value.dags.into_iter().map(|d| d.into()).collect(),
            total_entries: value.total_entries,
        }
    }
}

impl From<crate::airflow::model::v1::dagstats::DagStatistics> for DagStatistics {
    /// Converts a v1 dagstats::DagStatistics into the common DagStatistics model.
    ///
    /// # Examples
    ///
    /// ```
    /// let v1 = crate::airflow::model::v1::dagstats::DagStatistics {
    ///     dag_id: "example".into(),
    ///     stats: Vec::new(),
    /// };
    /// let common: crate::airflow::model::common::dagstats::DagStatistics = v1.into();
    /// assert_eq!(common.dag_id, "example");
    /// ```
    fn from(value: crate::airflow::model::v1::dagstats::DagStatistics) -> Self {
        DagStatistics {
            dag_id: value.dag_id,
            stats: value.stats.into_iter().map(|s| s.into()).collect(),
        }
    }
}

impl From<crate::airflow::model::v1::dagstats::DagStatistic> for DagStatistic {
    /// Converts a v1 `DagStatistic` into the common `DagStatistic`.
    ///
    /// # Examples
    ///
    /// ```
    /// let v1 = crate::airflow::model::v1::dagstats::DagStatistic {
    ///     state: "success".into(),
    ///     count: 3,
    /// };
    /// let common: crate::airflow::model::common::dagstats::DagStatistic = v1.into();
    /// assert_eq!(common.state, "success");
    /// assert_eq!(common.count, 3);
    /// ```
    fn from(value: crate::airflow::model::v1::dagstats::DagStatistic) -> Self {
        DagStatistic {
            state: value.state,
            count: value.count,
        }
    }
}

// From trait implementations for v2 models
impl From<crate::airflow::model::v2::dagstats::DagStatsResponse> for DagStatsResponse {
    /// Convert a v2 Airflow DAG stats response into the crate's common `DagStatsResponse`.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::airflow::model::v2::dagstats as v2;
    /// use crate::airflow::model::common::dagstats::DagStatsResponse as Common;
    ///
    /// let v2_resp = v2::DagStatsResponse { dags: vec![], total_entries: 0 };
    /// let common: Common = v2_resp.into();
    /// assert_eq!(common.total_entries, 0);
    /// ```
    fn from(value: crate::airflow::model::v2::dagstats::DagStatsResponse) -> Self {
        DagStatsResponse {
            dags: value.dags.into_iter().map(|d| d.into()).collect(),
            total_entries: value.total_entries,
        }
    }
}

impl From<crate::airflow::model::v2::dagstats::DagStatistics> for DagStatistics {
    /// Converts a v2 `DagStatistics` value into the common `DagStatistics` model.
    ///
    /// # Examples
    ///
    /// ```
    /// let v2 = crate::airflow::model::v2::dagstats::DagStatistics {
    ///     dag_id: "example_dag".to_string(),
    ///     stats: vec![],
    /// };
    /// let common: crate::airflow::model::common::dagstats::DagStatistics = v2.into();
    /// assert_eq!(common.dag_id, "example_dag");
    /// ```
    fn from(value: crate::airflow::model::v2::dagstats::DagStatistics) -> Self {
        DagStatistics {
            dag_id: value.dag_id,
            stats: value.stats.into_iter().map(|s| s.into()).collect(),
        }
    }
}

impl From<crate::airflow::model::v2::dagstats::DagStatistic> for DagStatistic {
    /// Converts a v2 `DagStatistic` into the common `DagStatistic`.
    ///
    /// # Examples
    ///
    /// ```
    /// let v2 = crate::airflow::model::v2::dagstats::DagStatistic {
    ///     state: "success".to_string(),
    ///     count: 3,
    /// };
    /// let common: crate::airflow::model::common::dagstats::DagStatistic = v2.into();
    /// assert_eq!(common.state, "success");
    /// assert_eq!(common.count, 3);
    /// ```
    fn from(value: crate::airflow::model::v2::dagstats::DagStatistic) -> Self {
        DagStatistic {
            state: value.state,
            count: value.count,
        }
    }
}