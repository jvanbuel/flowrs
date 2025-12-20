use anyhow::Result;
use async_trait::async_trait;
use log::debug;
use reqwest::{Method, Response};

use super::model;
use crate::airflow::{model::common::DagRunList, traits::DagRunOperations};

use super::V1Client;

#[async_trait]
impl DagRunOperations for V1Client {
    async fn list_dagruns(&self, dag_id: &str) -> Result<DagRunList> {
        let response: Response = self
            .base_api(Method::GET, &format!("dags/{dag_id}/dagRuns"))?
            .query(&[("order_by", "-execution_date"), ("limit", "50")])
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
            .base_api(Method::POST, "dags/~/dagRuns/list")?
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
        )?
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
        )?
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
            .base_api(Method::POST, &format!("dags/{dag_id}/dagRuns"))?
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        debug!("{resp:?}");
        Ok(())
    }
}
