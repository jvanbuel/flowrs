use anyhow::Result;
use async_trait::async_trait;
use log::debug;
use reqwest::{Method, Response};

use super::model;
use crate::airflow::model::common::dagrun::{DagRunState, RunType};
use crate::airflow::model::common::{DagRun, DagRunList};
use crate::airflow::traits::DagRunOperations;

use super::V2Client;

#[async_trait]
impl DagRunOperations for V2Client {
    async fn list_dagruns(&self, dag_id: &str) -> Result<DagRunList> {
        let response: Response = self
            .base_api(Method::GET, &format!("dags/{dag_id}/dagRuns"))
            .await?
            .query(&[("order_by", "-run_after"), ("limit", "50")])
            .send()
            .await?
            .error_for_status()?;
        let dagruns: model::dagrun::DagRunList =
            response.json::<model::dagrun::DagRunList>().await?;
        Ok(dagruns.into())
    }

    async fn list_all_dagruns(&self) -> Result<DagRunList> {
        let response: Response = self
            .base_api(Method::POST, "dags/~/dagRuns/list")
            .await?
            .json(&serde_json::json!({"page_limit": 200}))
            .send()
            .await?
            .error_for_status()?;
        let dagruns: model::dagrun::DagRunList =
            response.json::<model::dagrun::DagRunList>().await?;
        Ok(dagruns.into())
    }

    async fn mark_dag_run(&self, dag_id: &str, dag_run_id: &str, status: &str) -> Result<()> {
        self.base_api(
            Method::PATCH,
            &format!("dags/{dag_id}/dagRuns/{dag_run_id}"),
        )
        .await?
        .json(&serde_json::json!({"state": status}))
        .send()
        .await?
        .error_for_status()?;
        Ok(())
    }

    async fn clear_dagrun(&self, dag_id: &str, dag_run_id: &str) -> Result<()> {
        self.base_api(
            Method::POST,
            &format!("dags/{dag_id}/dagRuns/{dag_run_id}/clear"),
        )
        .await?
        .json(&serde_json::json!({"dry_run": false}))
        .send()
        .await?
        .error_for_status()?;
        Ok(())
    }

    async fn trigger_dag_run(&self, dag_id: &str, logical_date: Option<&str>) -> Result<()> {
        let body = serde_json::json!({"logical_date": logical_date});

        let resp: Response = self
            .base_api(Method::POST, &format!("dags/{dag_id}/dagRuns"))
            .await?
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        debug!("{resp:?}");
        Ok(())
    }
}

// From trait implementations for v2 models
impl From<model::dagrun::DagRun> for DagRun {
    fn from(value: model::dagrun::DagRun) -> Self {
        Self {
            dag_id: value.dag_id.into(),
            dag_run_id: value.dag_run_id.into(),
            logical_date: value.logical_date,
            data_interval_end: value.data_interval_end,
            data_interval_start: value.data_interval_start,
            end_date: value.end_date,
            start_date: value.start_date,
            last_scheduling_decision: value.last_scheduling_decision,
            run_type: RunType::from(value.run_type.as_str()),
            state: DagRunState::from(value.state.as_str()),
            note: value.note,
            external_trigger: None,
        }
    }
}

impl From<model::dagrun::DagRunList> for DagRunList {
    fn from(value: model::dagrun::DagRunList) -> Self {
        Self {
            dag_runs: value
                .dag_runs
                .into_iter()
                .map(std::convert::Into::into)
                .collect(),
            total_entries: value.total_entries,
        }
    }
}
