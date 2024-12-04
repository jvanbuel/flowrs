use crate::airflow::model::dagstats::DagStatsResponse;
use reqwest::Method;
use anyhow::Result;

use super::AirFlowClient;

impl AirFlowClient {
    pub async fn get_dag_stats(&self, dag_ids: Vec<&str>) -> Result<DagStatsResponse> {
        let response = self
            .base_api(Method::GET, "dagStats")?
            .query(&[("dag_ids", dag_ids.join(","))])
            .send()
            .await?;
        let dag_stats = response.json::<DagStatsResponse>().await?;
        Ok(dag_stats)
    }
}

#[cfg(test)]
mod tests {

    use super::AirFlowClient;
    use crate::airflow::config::FlowrsConfig;

    const TEST_CONFIG: &str = r#"[[servers]]
        name = "test"
        endpoint = "http://localhost:8080"

        [servers.auth.BasicAuth]
        username = "airflow"
        password = "airflow"
        "#;

    #[tokio::test]
    async fn test_get_dag_stats() {
        let config: FlowrsConfig = toml::from_str(str::trim(TEST_CONFIG)).unwrap();
        let client = AirFlowClient::new(config.servers.unwrap()[0].clone()).unwrap();

        let stats = client
            .get_dag_stats(vec!["example_bash_operator"])
            .await
            .unwrap();

        assert_eq!(stats.dags[0].dag_id, "example_bash_operator");
    }
}
