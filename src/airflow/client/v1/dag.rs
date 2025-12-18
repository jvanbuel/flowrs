use anyhow::Result;
use async_trait::async_trait;
use log::info;
use reqwest::Method;

use crate::airflow::{model::common::DagList, traits::DagOperations};

use super::model::dag::DagCollectionResponse;

use super::V1Client;

#[async_trait]
impl DagOperations for V1Client {
    async fn list_dags(&self) -> Result<DagList> {
        let r = self.base_api(Method::GET, "dags")?.build()?;
        let response = self.base.client.execute(r).await?.error_for_status()?;

        response
            .json::<DagCollectionResponse>()
            .await
            .map(|daglist| {
                info!("DAGs: {daglist:?}");
                daglist.into()
            })
            .map_err(std::convert::Into::into)
    }

    async fn toggle_dag(&self, dag_id: &str, is_paused: bool) -> Result<()> {
        self.base_api(Method::PATCH, &format!("dags/{dag_id}"))?
            .query(&[("update_mask", "is_paused")])
            .json(&serde_json::json!({"is_paused": !is_paused}))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    async fn get_dag_code(&self, dag: &crate::airflow::model::common::Dag) -> Result<String> {
        let r = self
            .base_api(Method::GET, &format!("dagSources/{}", dag.file_token))?
            .build()?;
        let response = self.base.client.execute(r).await?.error_for_status()?;
        let code = response.text().await?;
        Ok(code)
    }
}
