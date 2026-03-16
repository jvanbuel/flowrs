use anyhow::Result;
use log::debug;
use reqwest::{Method, Response};

use super::model;
use super::V2Client;

impl V2Client {
    pub async fn fetch_dagruns(&self, dag_id: &str) -> Result<model::dagrun::DagRunList> {
        let response: Response = self
            .base_api(Method::GET, &format!("dags/{dag_id}/dagRuns"))
            .await?
            .query(&[("order_by", "-run_after"), ("limit", "50")])
            .send()
            .await?
            .error_for_status()?;
        let dagruns: model::dagrun::DagRunList = response.json().await?;
        Ok(dagruns)
    }

    pub async fn fetch_all_dagruns(&self) -> Result<model::dagrun::DagRunList> {
        let response: Response = self
            .base_api(Method::POST, "dags/~/dagRuns/list")
            .await?
            .json(&serde_json::json!({"page_limit": 200}))
            .send()
            .await?
            .error_for_status()?;
        let dagruns: model::dagrun::DagRunList = response.json().await?;
        Ok(dagruns)
    }

    pub async fn patch_dag_run(&self, dag_id: &str, dag_run_id: &str, status: &str) -> Result<()> {
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

    pub async fn post_clear_dagrun(&self, dag_id: &str, dag_run_id: &str) -> Result<()> {
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

    pub async fn post_trigger_dag_run(
        &self,
        dag_id: &str,
        logical_date: Option<&str>,
        conf: Option<serde_json::Value>,
    ) -> Result<()> {
        let mut body = serde_json::json!({"logical_date": logical_date});
        if let Some(conf) = conf {
            body["conf"] = conf;
        }

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
