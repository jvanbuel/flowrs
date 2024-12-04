use anyhow::Result;
use log::error;
use reqwest::{Method, Response};

use super::AirFlowClient;
use crate::airflow::model::dag::DagList;

impl AirFlowClient {
    pub async fn list_dags(&self) -> Result<DagList> {
        let r = self.base_api(Method::GET, "dags")?.build()?;
        let response = self.client.execute(r).await?;

        let daglist = response.json::<DagList>().await;
        match daglist {
            Ok(daglist) => Ok(daglist),
            Err(e) => {
                error!("Error parsing response: {}", e);
                Ok(DagList::default())
            }
        }
    }

    // pub async fn get_dag_details(&self, dag_id: &str) -> Result<()> {
    //     let r = self
    //         .base_api(Method::GET, &format!("dags/{dag_id}/details"))?
    //         .build()?;
    //     let response = self.client.execute(r).await?;
    //     let _ = response.text().await?;
    //     Ok(())
    // }

    pub async fn toggle_dag(&self, dag_id: &str, is_paused: bool) -> Result<()> {
        let _: Response = self
            .base_api(Method::PATCH, &format!("dags/{dag_id}"))?
            .query(&[("update_mask", "is_paused")])
            .json(&serde_json::json!({"is_paused": !is_paused}))
            .send()
            .await?;
        Ok(())
    }

    pub async fn get_dag_code(&self, file_token: &String) -> Result<String> {
        let r = self
            .base_api(Method::GET, &format!("dagSources/{file_token}"))?
            .build()?;
        let response = self.client.execute(r).await?;
        let code = response.text().await?;
        Ok(code)
    }
}

#[cfg(test)]
mod tests {

    use super::AirFlowClient;
    use super::DagList;
    use crate::airflow::config::FlowrsConfig;

    const TEST_CONFIG: &str = r#"[[servers]]
        name = "test"
        endpoint = "http://localhost:8080"

        [servers.auth.BasicAuth]
        username = "airflow"
        password = "airflow"
        "#;

    #[tokio::test]
    async fn test_list_dags() {
        let config: FlowrsConfig = toml::from_str(str::trim(TEST_CONFIG)).unwrap();
        let client = AirFlowClient::new(config.servers.unwrap()[0].clone()).unwrap();
        let daglist: DagList = client.list_dags().await.unwrap();
        assert_eq!(daglist.dags[0].owners, vec!["airflow"]);
    }

    #[tokio::test]
    async fn test_get_dag_code() {
        let config: FlowrsConfig = toml::from_str(str::trim(TEST_CONFIG)).unwrap();
        let client = AirFlowClient::new(config.servers.unwrap()[0].clone()).unwrap();

        let dag = client.list_dags().await.unwrap().dags[0].clone();
        let code = client.get_dag_code(&dag.file_token).await.unwrap();
        assert!(code.contains("with DAG"));
    }
}
