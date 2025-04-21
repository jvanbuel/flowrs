use reqwest::Method;

use crate::airflow::model::log::Log;
use anyhow::Result;

use super::AirFlowClient;

impl AirFlowClient {
    pub async fn get_task_logs(
        &self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
        task_try: u16,
    ) -> Result<Log> {
        let reponse = self
            .base_api(
                Method::GET,
                format!(
                    "dags/{dag_id}/dagRuns/{dag_run_id}/taskInstances/{task_id}/logs/{task_try}",
                )
                .as_str(),
            )?
            .query(&[("full_content", "true")])
            .header("Accept", "application/json") // Important, otherwise will return plain text
            .send()
            .await?;
        let log = reponse.json::<Log>().await?;
        Ok(log)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airflow::config::{AirflowAuth, AirflowConfig, BasicAuth};
    use crate::airflow::model::log::Log;
    use mockito::Mock;
    use serde_json::json;

    #[tokio::test]
    async fn test_get_task_logs() {
        let mut server = mockito::Server::new_async().await;

        let dag_id = "dag_id";
        let dag_run_id = "dag_run_id";
        let task_id = "task_id";
        let task_try = 1;

        let _: Mock = server
            .mock(
                "GET",
                format!(
                    "/api/v1/dags/{dag_id}/dagRuns/{dag_run_id}/taskInstances/{task_id}/logs/{task_try}"
                )
                .as_str(),
            )
            .with_status(200)
            .with_body(
                json!(Log {
                    continuation_token: None,
                    content: "task_log".to_string(),
                })
                .to_string(),
            )
            .create_async()
            .await;

        let airflow_config = AirflowConfig {
            name: "test".to_string(),
            endpoint: server.url(),
            auth: AirflowAuth::Basic(BasicAuth {
                username: "airflow".to_string(),
                password: "airflow".to_string(),
            }),
            managed: None,
        };

        let client = AirFlowClient::new(airflow_config).unwrap();

        let log: Log = client
            .get_task_logs(dag_id, dag_run_id, task_id, task_try)
            .await
            .unwrap();

        assert!(log.content == "task_log");
    }
}
