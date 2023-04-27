use std::error::Error;

use reqwest::{Method, Response};

use crate::model::dagrun::DagRunList;

use super::client::AirFlowClient;

impl AirFlowClient {
    pub async fn list_dagruns(
        &self,
        dag_id: &str,
    ) -> Result<DagRunList, Box<dyn Error + Send + Sync>> {
        let response: Response = self
            .base_api(Method::GET, format!("dags/{dag_id}/dagRuns").as_str())?
            .send()
            .await?;
        let dagruns: DagRunList = response.json::<DagRunList>().await?;
        Ok(dagruns)
    }

    pub async fn list_all_dagruns(&self) -> Result<DagRunList, Box<dyn Error + Send + Sync>> {
        let response: Response = self
            .base_api(Method::POST, "dags/~/dagRuns/list")?
            .json(&serde_json::json!({"page_limit": 100}))
            .send()
            .await?;
        let dagruns: DagRunList = response.json::<DagRunList>().await?;
        Ok(dagruns)
    }

    pub async fn clear_dagrun(
        &self,
        dag_id: &str,
        dag_run_id: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let _: Response = self
            .base_api(
                Method::POST,
                format!("dags/{dag_id}/dagRuns/{dag_run_id}/clear").as_str(),
            )?
            .send()
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::app::auth::get_config;
    use crate::app::client::AirFlowClient;
    use crate::app::dagruns::DagRunList;

    #[tokio::test]
    async fn test_list_dags() {
        let binding = get_config(Some(&Path::new(".flowrs")));
        let server = binding.servers[1].clone();
        let client = AirFlowClient::new(server);
        let first_dag = &client.list_dags().await.unwrap().dags[0];

        println!("{:?}", client.config);
        let dagrun_list: DagRunList = client
            .list_dagruns(first_dag.dag_id.as_str())
            .await
            .unwrap();
        assert_eq!(dagrun_list.dag_runs.len() >= 1, true);
    }

    #[tokio::test]
    async fn test_list_all_dags() {
        let binding = get_config(Some(&Path::new(".flowrs")));
        let server = binding.servers[1].clone();
        let client = AirFlowClient::new(server);

        let dagrun_list: DagRunList = client.list_all_dagruns().await.unwrap();
        assert_eq!(dagrun_list.dag_runs.len() >= 1, true);
    }
}
