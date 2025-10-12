use anyhow::Result;
use async_trait::async_trait;
use log::info;
use reqwest::Method;

use crate::airflow::{
    model::{common::DagList, v1::dag::DagCollectionResponse},
    traits::DagOperations,
};

use super::V1Client;

#[async_trait]
impl DagOperations for V1Client {
    /// Fetches the list of DAGs from the Airflow V1 API.
    ///
    /// # Returns
    ///
    /// `DagList` containing DAG metadata retrieved from the server.
    ///
    /// # Examples
    ///
    /// ```
    /// #[tokio::test]
    /// async fn example_list_dags() -> anyhow::Result<()> {
    ///     let client = get_test_client(); // test helper that constructs a V1Client
    ///     let dags = client.list_dags().await?;
    ///     assert!(!dags.items.is_empty());
    ///     Ok(())
    /// }
    /// ```
    async fn list_dags(&self) -> Result<DagList> {
        let r = self.base_api(Method::GET, "dags")?.build()?;
        let response = self.base.client.execute(r).await?;

        response
            .json::<DagCollectionResponse>()
            .await
            .map(|daglist| {
                info!("DAGs: {:?}", daglist);
                daglist.into()
            })
            .map_err(|e| e.into())
    }

    /// Toggles a DAG's pause state by sending an update that sets `is_paused` to the opposite of `is_paused`.
    ///
    /// Updates the DAG identified by `dag_id` via a PATCH request that modifies the `is_paused` field.
    ///
    /// # Arguments
    ///
    /// - `dag_id`: Identifier of the DAG to update.
    /// - `is_paused`: Current pause state of the DAG; the function will set `is_paused` to the opposite value.
    ///
    /// # Returns
    ///
    /// `()` on success.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example(client: &crate::V1Client) -> anyhow::Result<()> {
    /// client.toggle_dag("example_dag", true).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn toggle_dag(&self, dag_id: &str, is_paused: bool) -> Result<()> {
        let _ = self
            .base_api(Method::PATCH, &format!("dags/{dag_id}"))?
            .query(&[("update_mask", "is_paused")])
            .json(&serde_json::json!({"is_paused": !is_paused}))
            .send()
            .await?;
        Ok(())
    }

    /// Retrieves the source code of a DAG file identified by `file_token`.
    ///
    /// On success, returns the DAG file contents as a `String`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use anyhow::Result;
    /// # async fn example(client: &crate::V1Client) -> Result<()> {
    /// let code = client.get_dag_code("example_file_token").await?;
    /// assert!(code.contains("with DAG") || !code.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    async fn get_dag_code(&self, file_token: &str) -> Result<String> {
        let r = self
            .base_api(Method::GET, &format!("dagSources/{file_token}"))?
            .build()?;
        let response = self.base.client.execute(r).await?;
        let code = response.text().await?;
        Ok(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airflow::client::base::BaseClient;

    const TEST_CONFIG: &str = r#"[[servers]]
        name = "test"
        endpoint = "http://localhost:8080"

        [servers.auth.Basic]
        username = "airflow"
        password = "airflow"
        "#;

    /// Constructs a V1Client configured for tests from the embedded TEST_CONFIG.
    ///
    /// The client is created by parsing the test FlowrsConfig and using its first server entry.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = get_test_client();
    /// // use `client` in async test helpers or to call V1Client methods
    /// ```
    fn get_test_client() -> V1Client {
        let config: crate::airflow::config::FlowrsConfig =
            toml::from_str(TEST_CONFIG.trim()).unwrap();
        let base = BaseClient::new(config.servers.unwrap()[0].clone()).unwrap();
        V1Client::new(base)
    }

    #[tokio::test]
    async fn test_list_dags() {
        let client = get_test_client();
        let daglist: DagList = client.list_dags().await.unwrap();
        assert_eq!(daglist.dags[0].owners, vec!["airflow"]);
    }

    #[tokio::test]
    async fn test_get_dag_code() {
        let client = get_test_client();
        let dag = client.list_dags().await.unwrap().dags[0].clone();
        let code = client.get_dag_code(&dag.file_token).await.unwrap();
        assert!(code.contains("with DAG"));
    }
}