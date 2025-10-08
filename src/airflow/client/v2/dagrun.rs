use anyhow::Result;
use async_trait::async_trait;
use log::debug;
use reqwest::{Method, Response};

use crate::airflow::{model::common::DagRunList, model::v2, traits::DagRunOperations};

use super::V2Client;

#[async_trait]
impl DagRunOperations for V2Client {
    async fn list_dagruns(&self, dag_id: &str) -> Result<DagRunList> {
        let response: Response = self
            .base_api(Method::GET, &format!("dags/{dag_id}/dagRuns"))?
            .query(&[("order_by", "-logical_date")])
            .send()
            .await?;
        let dagruns: v2::dagrun::DagRunList = response.json::<v2::dagrun::DagRunList>().await?;
        Ok(dagruns.into())
    }

    async fn list_all_dagruns(&self) -> Result<DagRunList> {
        let response: Response = self
            .base_api(Method::POST, "dags/~/dagRuns/list")?
            .json(&serde_json::json!({"page_limit": 200}))
            .send()
            .await?;
        let dagruns: v2::dagrun::DagRunList = response.json::<v2::dagrun::DagRunList>().await?;
        Ok(dagruns.into())
    }

    async fn mark_dag_run(&self, dag_id: &str, dag_run_id: &str, status: &str) -> Result<()> {
        let _: Response = self
            .base_api(
                Method::PATCH,
                &format!("dags/{dag_id}/dagRuns/{dag_run_id}"),
            )?
            .json(&serde_json::json!({"state": status}))
            .send()
            .await?;
        Ok(())
    }

    async fn clear_dagrun(&self, dag_id: &str, dag_run_id: &str) -> Result<()> {
        let _: Response = self
            .base_api(
                Method::POST,
                &format!("dags/{dag_id}/dagRuns/{dag_run_id}/clear"),
            )?
            .json(&serde_json::json!({"dry_run": false}))
            .send()
            .await?;
        Ok(())
    }

    async fn trigger_dag_run(&self, dag_id: &str) -> Result<()> {
        let resp: Response = self
            .base_api(Method::POST, &format!("dags/{dag_id}/dagRuns"))?
            .json(&serde_json::json!({}))
            .send()
            .await?;
        debug!("{:?}", resp);
        Ok(())
    }
}
