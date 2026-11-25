use log::debug;
use reqwest::Method;

use super::model;
use super::V2Client;
use crate::client::{read_json, task_instance_path};
use crate::error::Result;

impl V2Client {
    pub async fn fetch_task_logs(
        &self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
        map_index: i64,
        task_try: u32,
    ) -> Result<model::log::Log> {
        // Logs take `map_index` as a query parameter, not a path segment.
        let path = task_instance_path(dag_id, dag_run_id, task_id, -1);
        let mut request = self
            .base_api(Method::GET, &format!("{path}/logs/{task_try}"))
            .await?
            .query(&[("full_content", "true")]);
        if map_index >= 0 {
            request = request.query(&[("map_index", map_index)]);
        }
        let request = request.header("Accept", "application/json");
        let response = self.execute(request).await?;

        debug!("Response: {response:?}");
        let log: model::log::Log = read_json(response, "task logs response").await?;
        debug!("Parsed Log: {log:?}");
        Ok(log)
    }
}
