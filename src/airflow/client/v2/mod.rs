pub mod model;

mod dag;
mod dagrun;
mod dagstats;
mod log;
mod task;
mod taskinstance;

use anyhow::Result;
use reqwest::Method;
use url::{form_urlencoded, Url};

use super::base::BaseClient;
use crate::airflow::{config::AirflowVersion, traits::AirflowClient};
use crate::app::worker::OpenItem;

/// API v2 client implementation (for Airflow v3, uses /api/v2 endpoint)
#[derive(Debug, Clone)]
pub struct V2Client {
    base: BaseClient,
}

impl V2Client {
    const API_VERSION: &'static str = "api/v2";

    pub fn new(base: BaseClient) -> Self {
        Self { base }
    }

    fn base_api(&self, method: Method, endpoint: &str) -> Result<reqwest::RequestBuilder> {
        self.base.base_api(method, endpoint, Self::API_VERSION)
    }
}

impl AirflowClient for V2Client {
    fn get_version(&self) -> AirflowVersion {
        AirflowVersion::V3
    }

    fn build_open_url(&self, item: &OpenItem) -> Result<String> {
        let mut base_url = Url::parse(&self.base.config.endpoint)?;

        match item {
            OpenItem::Config(config_endpoint) => {
                base_url = config_endpoint.parse()?;
            }
            OpenItem::Dag { dag_id } => {
                base_url = base_url.join(&format!("dags/{dag_id}"))?;
            }
            OpenItem::DagRun { dag_id, dag_run_id } => {
                let escaped_dag_run_id: String =
                    form_urlencoded::byte_serialize(dag_run_id.as_bytes()).collect();
                base_url = base_url.join(&format!("dags/{dag_id}/runs/{escaped_dag_run_id}"))?;
            }
            OpenItem::TaskInstance {
                dag_id,
                dag_run_id,
                task_id,
            } => {
                let escaped_dag_run_id: String =
                    form_urlencoded::byte_serialize(dag_run_id.as_bytes()).collect();
                let escaped_task_id: String =
                    form_urlencoded::byte_serialize(task_id.as_bytes()).collect();
                base_url = base_url.join(&format!(
                    "dags/{dag_id}/runs/{escaped_dag_run_id}/tasks/{escaped_task_id}"
                ))?;
            }
            OpenItem::Log {
                dag_id,
                dag_run_id,
                task_id,
                task_try,
            } => {
                let escaped_dag_run_id: String =
                    form_urlencoded::byte_serialize(dag_run_id.as_bytes()).collect();
                let escaped_task_id: String =
                    form_urlencoded::byte_serialize(task_id.as_bytes()).collect();
                base_url = base_url.join(&format!(
                    "dags/{dag_id}/runs/{escaped_dag_run_id}/tasks/{escaped_task_id}"
                ))?;
                base_url.set_query(Some(&format!("tab=logs&try_number={task_try}")));
            }
        }

        Ok(base_url.to_string())
    }
}
