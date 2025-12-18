use anyhow::Result;
use async_trait::async_trait;
use reqwest::Method;

use super::model;
use crate::airflow::{model::common::DagList, traits::DagOperations};

use super::V2Client;

#[async_trait]
impl DagOperations for V2Client {
    async fn list_dags(&self) -> Result<DagList> {
        let r = self.base_api(Method::GET, "dags")?.build()?;
        let response = self.base.client.execute(r).await?.error_for_status()?;

        response
            .json::<model::dag::DagList>()
            .await
            .map(std::convert::Into::into)
            .map_err(std::convert::Into::into)
    }

    async fn toggle_dag(&self, dag_id: &str, is_paused: bool) -> Result<()> {
        self.base_api(Method::PATCH, &format!("dags/{dag_id}"))?
            .json(&serde_json::json!({"is_paused": !is_paused}))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    async fn get_dag_code(&self, dag: &crate::airflow::model::common::Dag) -> Result<String> {
        let r = self
            .base_api(Method::GET, &format!("dagSources/{}", dag.dag_id))?
            .build()?;
        let response = self.base.client.execute(r).await?.error_for_status()?;
        let dag_source: model::dag::DagSource = response.json().await?;
        Ok(dag_source.content)
    }
}
