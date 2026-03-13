use anyhow::Result;
use url::{form_urlencoded, Url};

use crate::airflow::model::common::OpenItem;

pub(super) fn build_v1_open_url(endpoint: &str, item: &OpenItem) -> Result<String> {
    let mut base_url = Url::parse(endpoint)?;

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
            base_url = base_url.join(&format!("dags/{dag_id}/grid"))?;
            base_url.set_query(Some(&format!("dag_run_id={escaped_dag_run_id}")));
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
            base_url = base_url.join(&format!("dags/{dag_id}/grid"))?;
            base_url.set_query(Some(&format!(
                "dag_run_id={escaped_dag_run_id}&task_id={escaped_task_id}"
            )));
        }
        OpenItem::Log {
            dag_id,
            dag_run_id,
            task_id,
            task_try: _,
        } => {
            let escaped_dag_run_id: String =
                form_urlencoded::byte_serialize(dag_run_id.as_bytes()).collect();
            let escaped_task_id: String =
                form_urlencoded::byte_serialize(task_id.as_bytes()).collect();
            base_url = base_url.join(&format!("dags/{dag_id}/grid"))?;
            base_url.set_query(Some(&format!(
                "dag_run_id={escaped_dag_run_id}&task_id={escaped_task_id}&tab=logs"
            )));
        }
    }

    Ok(base_url.to_string())
}

pub(super) fn build_v2_open_url(endpoint: &str, item: &OpenItem) -> Result<String> {
    let mut base_url = Url::parse(endpoint)?;

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
