use anyhow::Result;
use async_trait::async_trait;
use log::debug;
use reqwest::{Method, Response};

use super::model;
use crate::airflow::{
    model::common::DagRunList,
    traits::{DagRunDateFilter, DagRunOperations},
};

use super::V1Client;

#[async_trait]
impl DagRunOperations for V1Client {
    async fn list_dagruns(
        &self,
        dag_id: &str,
        date_filter: &DagRunDateFilter,
    ) -> Result<DagRunList> {
        let mut query: Vec<(&str, String)> = vec![
            ("order_by", "-execution_date".to_string()),
            ("limit", "50".to_string()),
        ];
        if let Some(start) = date_filter.start_date {
            query.push(("execution_date_gte", format!("{start}")));
        }
        if let Some(end) = date_filter.end_date {
            // Add one day to make the end date inclusive
            if let Some(next_day) = end.next_day() {
                query.push(("execution_date_lte", format!("{next_day}")));
            }
        }
        let response: Response = self
            .base_api(Method::GET, &format!("dags/{dag_id}/dagRuns"))
            .await?
            .query(&query)
            .send()
            .await?
            .error_for_status()?;

        let dagruns: model::dagrun::DAGRunCollectionResponse = response
            .json::<model::dagrun::DAGRunCollectionResponse>()
            .await?;
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
        let dagruns: model::dagrun::DAGRunCollectionResponse = response
            .json::<model::dagrun::DAGRunCollectionResponse>()
            .await?;
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
        // Somehow Airflow V1 API does not accept null for logical_date
        let body = logical_date.map_or_else(
            || serde_json::json!({}),
            |date| serde_json::json!({ "logical_date": date }),
        );

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
