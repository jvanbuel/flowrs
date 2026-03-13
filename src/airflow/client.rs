use anyhow::Result;
use async_trait::async_trait;
use url::{form_urlencoded, Url};

use flowrs_airflow::client::{BaseClient, V1Client, V2Client};
use flowrs_airflow::{AirflowConfig, AirflowVersion};

use super::model::common::dagrun::{DagRunState, RunType};
use super::model::common::dagstats::{DagStatistic, DagStatistics};
use super::model::common::taskinstance::TaskInstanceState;
use super::model::common::{
    Dag, DagList, DagRun, DagRunList, DagStatsResponse, Log, OpenItem, Tag, Task, TaskInstance,
    TaskInstanceList, TaskList, TaskTryGantt,
};
use super::traits::{
    AirflowClient, DagOperations, DagRunOperations, DagStatsOperations, LogOperations,
    TaskInstanceOperations, TaskOperations,
};

/// Wrapper enum that owns a versioned Airflow HTTP client and implements the TUI trait layer.
pub enum FlowrsClient {
    V1(V1Client),
    V2(V2Client),
}

impl FlowrsClient {
    /// Create a new `FlowrsClient` from an `AirflowConfig`.
    pub fn new(config: &AirflowConfig) -> Result<Self> {
        let base = BaseClient::new(config.clone())?;
        match config.version {
            AirflowVersion::V2 => Ok(Self::V1(V1Client::new(base))),
            AirflowVersion::V3 => Ok(Self::V2(V2Client::new(base))),
        }
    }
}

// ---- AirflowClient (super-trait) ----

impl AirflowClient for FlowrsClient {
    fn get_version(&self) -> AirflowVersion {
        match self {
            Self::V1(_) => AirflowVersion::V2,
            Self::V2(_) => AirflowVersion::V3,
        }
    }

    fn build_open_url(&self, item: &OpenItem) -> Result<String> {
        match self {
            Self::V1(client) => build_v1_open_url(client.endpoint(), item),
            Self::V2(client) => build_v2_open_url(client.endpoint(), item),
        }
    }
}

// ---- DagOperations ----

#[async_trait]
impl DagOperations for FlowrsClient {
    async fn list_dags(&self) -> Result<DagList> {
        match self {
            Self::V1(client) => {
                let response = client.fetch_dags().await?;
                Ok(v1_dag_collection_to_dag_list(response))
            }
            Self::V2(client) => {
                let response = client.fetch_dags().await?;
                Ok(v2_dag_list_to_dag_list(response))
            }
        }
    }

    async fn toggle_dag(&self, dag_id: &str, is_paused: bool) -> Result<()> {
        match self {
            Self::V1(client) => client.patch_dag_pause(dag_id, is_paused).await,
            Self::V2(client) => client.patch_dag_pause(dag_id, is_paused).await,
        }
    }

    async fn get_dag_code(&self, dag: &Dag) -> Result<String> {
        match self {
            Self::V1(client) => client.fetch_dag_code(&dag.file_token).await,
            Self::V2(client) => client.fetch_dag_code(&dag.dag_id).await,
        }
    }
}

// ---- DagRunOperations ----

#[async_trait]
impl DagRunOperations for FlowrsClient {
    async fn list_dagruns(&self, dag_id: &str) -> Result<DagRunList> {
        match self {
            Self::V1(client) => {
                let response = client.fetch_dagruns(dag_id).await?;
                Ok(v1_dagrun_collection_to_list(response))
            }
            Self::V2(client) => {
                let response = client.fetch_dagruns(dag_id).await?;
                Ok(v2_dagrun_list_to_list(response))
            }
        }
    }

    async fn list_all_dagruns(&self) -> Result<DagRunList> {
        match self {
            Self::V1(client) => {
                let response = client.fetch_all_dagruns().await?;
                Ok(v1_dagrun_collection_to_list(response))
            }
            Self::V2(client) => {
                let response = client.fetch_all_dagruns().await?;
                Ok(v2_dagrun_list_to_list(response))
            }
        }
    }

    async fn mark_dag_run(&self, dag_id: &str, dag_run_id: &str, status: &str) -> Result<()> {
        match self {
            Self::V1(client) => client.patch_dag_run(dag_id, dag_run_id, status).await,
            Self::V2(client) => client.patch_dag_run(dag_id, dag_run_id, status).await,
        }
    }

    async fn clear_dagrun(&self, dag_id: &str, dag_run_id: &str) -> Result<()> {
        match self {
            Self::V1(client) => client.post_clear_dagrun(dag_id, dag_run_id).await,
            Self::V2(client) => client.post_clear_dagrun(dag_id, dag_run_id).await,
        }
    }

    async fn trigger_dag_run(&self, dag_id: &str, logical_date: Option<&str>) -> Result<()> {
        match self {
            Self::V1(client) => client.post_trigger_dag_run(dag_id, logical_date).await,
            Self::V2(client) => client.post_trigger_dag_run(dag_id, logical_date).await,
        }
    }
}

// ---- TaskInstanceOperations ----

#[async_trait]
impl TaskInstanceOperations for FlowrsClient {
    async fn list_task_instances(
        &self,
        dag_id: &str,
        dag_run_id: &str,
    ) -> Result<TaskInstanceList> {
        match self {
            Self::V1(client) => {
                let response = client.fetch_task_instances(dag_id, dag_run_id).await?;
                Ok(v1_task_instance_collection_to_list(response))
            }
            Self::V2(client) => {
                let response = client.fetch_task_instances(dag_id, dag_run_id).await?;
                Ok(v2_task_instance_list_to_list(response))
            }
        }
    }

    async fn list_all_taskinstances(&self) -> Result<TaskInstanceList> {
        match self {
            Self::V1(client) => {
                let response = client.fetch_all_task_instances().await?;
                Ok(v1_task_instance_collection_to_list(response))
            }
            Self::V2(client) => {
                let response = client.fetch_all_task_instances().await?;
                Ok(v2_task_instance_list_to_list(response))
            }
        }
    }

    async fn list_task_instance_tries(
        &self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
    ) -> Result<Vec<TaskTryGantt>> {
        match self {
            Self::V1(client) => {
                let response = client
                    .fetch_task_instance_tries(dag_id, dag_run_id, task_id)
                    .await?;
                Ok(response
                    .task_instances
                    .into_iter()
                    .map(v1_task_instance_try_to_gantt)
                    .collect())
            }
            Self::V2(client) => {
                let response = client
                    .fetch_task_instance_tries(dag_id, dag_run_id, task_id)
                    .await?;
                Ok(response
                    .task_instances
                    .into_iter()
                    .map(v2_task_instance_try_to_gantt)
                    .collect())
            }
        }
    }

    async fn mark_task_instance(
        &self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
        status: &str,
    ) -> Result<()> {
        match self {
            Self::V1(client) => {
                client
                    .patch_task_instance(dag_id, dag_run_id, task_id, status)
                    .await
            }
            Self::V2(client) => {
                client
                    .patch_task_instance(dag_id, dag_run_id, task_id, status)
                    .await
            }
        }
    }

    async fn clear_task_instance(
        &self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
    ) -> Result<()> {
        match self {
            Self::V1(client) => {
                client
                    .post_clear_task_instance(dag_id, dag_run_id, task_id)
                    .await
            }
            Self::V2(client) => {
                client
                    .post_clear_task_instance(dag_id, dag_run_id, task_id)
                    .await
            }
        }
    }
}

// ---- LogOperations ----

#[async_trait]
impl LogOperations for FlowrsClient {
    async fn get_task_logs(
        &self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
        task_try: u32,
    ) -> Result<Log> {
        match self {
            Self::V1(client) => {
                let response = client
                    .fetch_task_logs(dag_id, dag_run_id, task_id, task_try)
                    .await?;
                Ok(v1_log_to_log(response))
            }
            Self::V2(client) => {
                let response = client
                    .fetch_task_logs(dag_id, dag_run_id, task_id, task_try)
                    .await?;
                Ok(v2_log_to_log(response))
            }
        }
    }
}

// ---- DagStatsOperations ----

#[async_trait]
impl DagStatsOperations for FlowrsClient {
    async fn get_dag_stats(&self, dag_ids: Vec<&str>) -> Result<DagStatsResponse> {
        match self {
            Self::V1(client) => {
                let response = client.fetch_dag_stats(dag_ids).await?;
                Ok(v1_dagstats_to_response(response))
            }
            Self::V2(client) => {
                let response = client.fetch_dag_stats(dag_ids).await?;
                Ok(v2_dagstats_to_response(response))
            }
        }
    }
}

// ---- TaskOperations ----

#[async_trait]
impl TaskOperations for FlowrsClient {
    async fn list_tasks(&self, dag_id: &str) -> Result<TaskList> {
        match self {
            Self::V1(client) => {
                let response = client.fetch_tasks(dag_id).await?;
                Ok(v1_task_collection_to_list(response))
            }
            Self::V2(client) => {
                let response = client.fetch_tasks(dag_id).await?;
                Ok(v2_task_collection_to_list(response))
            }
        }
    }
}

// =============================================================================
// V1 conversion functions
// =============================================================================

fn v1_dag_to_dag(value: flowrs_airflow::client::v1::model::dag::DagResponse) -> Dag {
    Dag {
        dag_id: value.dag_id.into(),
        dag_display_name: Some(value.dag_display_name),
        description: value.description,
        fileloc: value.fileloc,
        is_paused: value.is_paused.unwrap_or(false),
        is_active: value.is_active,
        has_import_errors: value.has_import_errors.unwrap_or(false),
        has_task_concurrency_limits: value.has_task_concurrency_limits.unwrap_or(false),
        last_parsed_time: value.last_parsed_time,
        last_expired: value.last_expired,
        max_active_tasks: value.max_active_tasks.unwrap_or(0),
        max_active_runs: value.max_active_runs,
        next_dagrun_logical_date: value.next_dagrun,
        next_dagrun_data_interval_start: value.next_dagrun_data_interval_start,
        next_dagrun_data_interval_end: value.next_dagrun_data_interval_end,
        next_dagrun_create_after: value.next_dagrun_create_after,
        owners: value.owners.clone(),
        tags: value
            .tags
            .unwrap_or_default()
            .into_iter()
            .map(|t| Tag { name: t.name })
            .collect(),
        file_token: value.file_token.clone(),
        timetable_description: value.timetable_description,
    }
}

fn v1_dag_collection_to_dag_list(
    value: flowrs_airflow::client::v1::model::dag::DagCollectionResponse,
) -> DagList {
    DagList {
        dags: value.dags.into_iter().map(v1_dag_to_dag).collect(),
        total_entries: value.total_entries,
    }
}

fn v1_dagrun_to_dagrun(value: flowrs_airflow::client::v1::model::dagrun::DAGRunResponse) -> DagRun {
    DagRun {
        dag_id: value.dag_id.into(),
        dag_run_id: value.dag_run_id.unwrap_or_default().into(),
        logical_date: value.logical_date,
        data_interval_end: value.data_interval_end,
        data_interval_start: value.data_interval_start,
        end_date: value.end_date,
        start_date: value.start_date,
        last_scheduling_decision: value.last_scheduling_decision,
        run_type: RunType::from(value.run_type.as_str()),
        state: DagRunState::from(value.state.as_str()),
        note: value.note,
        external_trigger: Some(value.external_trigger),
    }
}

fn v1_dagrun_collection_to_list(
    value: flowrs_airflow::client::v1::model::dagrun::DAGRunCollectionResponse,
) -> DagRunList {
    DagRunList {
        dag_runs: value
            .dag_runs
            .into_iter()
            .map(v1_dagrun_to_dagrun)
            .collect(),
        total_entries: value.total_entries,
    }
}

fn v1_task_instance_to_domain(
    value: flowrs_airflow::client::v1::model::taskinstance::TaskInstanceResponse,
) -> TaskInstance {
    TaskInstance {
        task_id: value.task_id.into(),
        dag_id: value.dag_id.into(),
        dag_run_id: value.dag_run_id.into(),
        logical_date: Some(value.execution_date),
        start_date: value.start_date,
        end_date: value.end_date,
        duration: value.duration,
        state: value.state.map(|s| TaskInstanceState::from(s.as_str())),
        try_number: value.try_number,
        max_tries: value.max_tries,
        map_index: value.map_index,
        hostname: Some(value.hostname),
        unixname: Some(value.unixname),
        pool: value.pool,
        pool_slots: value.pool_slots,
        queue: value.queue,
        priority_weight: value.priority_weight,
        operator: value.operator,
        queued_when: value.queued_when,
        pid: value.pid,
        note: value.note,
    }
}

fn v1_task_instance_collection_to_list(
    value: flowrs_airflow::client::v1::model::taskinstance::TaskInstanceCollectionResponse,
) -> TaskInstanceList {
    TaskInstanceList {
        task_instances: value
            .task_instances
            .into_iter()
            .map(v1_task_instance_to_domain)
            .collect(),
        total_entries: value.total_entries,
    }
}

fn v1_task_instance_try_to_gantt(
    value: flowrs_airflow::client::v1::model::taskinstance::TaskInstanceTryResponse,
) -> TaskTryGantt {
    TaskTryGantt {
        try_number: value.try_number,
        start_date: value.start_date,
        end_date: value.end_date,
        state: value.state.map(|s| TaskInstanceState::from(s.as_str())),
    }
}

fn v1_task_to_task(value: flowrs_airflow::client::v1::model::task::TaskResponse) -> Task {
    Task {
        task_id: value.task_id,
        downstream_task_ids: value.downstream_task_ids,
    }
}

fn v1_task_collection_to_list(
    value: flowrs_airflow::client::v1::model::task::TaskCollectionResponse,
) -> TaskList {
    TaskList {
        tasks: value.tasks.into_iter().map(v1_task_to_task).collect(),
    }
}

fn v1_log_to_log(value: flowrs_airflow::client::v1::model::log::Log) -> Log {
    Log {
        continuation_token: value.continuation_token,
        content: flowrs_airflow::client::v1::log::parse_v1_log_content(&value.content),
    }
}

fn v1_dagstats_to_response(
    value: flowrs_airflow::client::v1::model::dagstats::DagStatsResponse,
) -> DagStatsResponse {
    DagStatsResponse {
        dags: value
            .dags
            .into_iter()
            .map(|ds| DagStatistics {
                dag_id: ds.dag_id,
                stats: ds
                    .stats
                    .into_iter()
                    .map(|s| DagStatistic {
                        state: DagRunState::from(s.state.as_str()),
                        count: s.count,
                    })
                    .collect(),
            })
            .collect(),
        total_entries: value.total_entries,
    }
}

// =============================================================================
// V2 conversion functions
// =============================================================================

fn v2_dag_to_dag(value: flowrs_airflow::client::v2::model::dag::Dag) -> Dag {
    Dag {
        dag_id: value.dag_id.into(),
        dag_display_name: Some(value.dag_display_name),
        description: value.description,
        fileloc: value.fileloc,
        is_paused: value.is_paused,
        is_active: None,
        has_import_errors: value.has_import_errors,
        has_task_concurrency_limits: value.has_task_concurrency_limits,
        last_parsed_time: value.last_parsed_time,
        last_expired: value.last_expired,
        max_active_tasks: value.max_active_tasks,
        max_active_runs: value.max_active_runs,
        next_dagrun_logical_date: value.next_dagrun_logical_date,
        next_dagrun_create_after: value.next_dagrun_run_after,
        next_dagrun_data_interval_start: value.next_dagrun_data_interval_start,
        next_dagrun_data_interval_end: value.next_dagrun_data_interval_end,
        owners: value.owners,
        tags: value
            .tags
            .into_iter()
            .map(|t| Tag { name: t.name })
            .collect(),
        file_token: value.file_token,
        timetable_description: value.timetable_description,
    }
}

fn v2_dag_list_to_dag_list(value: flowrs_airflow::client::v2::model::dag::DagList) -> DagList {
    DagList {
        dags: value.dags.into_iter().map(v2_dag_to_dag).collect(),
        total_entries: value.total_entries,
    }
}

fn v2_dagrun_to_dagrun(value: flowrs_airflow::client::v2::model::dagrun::DagRun) -> DagRun {
    DagRun {
        dag_id: value.dag_id.into(),
        dag_run_id: value.dag_run_id.into(),
        logical_date: value.logical_date,
        data_interval_end: value.data_interval_end,
        data_interval_start: value.data_interval_start,
        end_date: value.end_date,
        start_date: value.start_date,
        last_scheduling_decision: value.last_scheduling_decision,
        run_type: RunType::from(value.run_type.as_str()),
        state: DagRunState::from(value.state.as_str()),
        note: value.note,
        external_trigger: None,
    }
}

fn v2_dagrun_list_to_list(
    value: flowrs_airflow::client::v2::model::dagrun::DagRunList,
) -> DagRunList {
    DagRunList {
        dag_runs: value
            .dag_runs
            .into_iter()
            .map(v2_dagrun_to_dagrun)
            .collect(),
        total_entries: value.total_entries,
    }
}

fn v2_task_instance_to_domain(
    value: flowrs_airflow::client::v2::model::taskinstance::TaskInstance,
) -> TaskInstance {
    TaskInstance {
        task_id: value.task_id.into(),
        dag_id: value.dag_id.into(),
        dag_run_id: value.dag_run_id.into(),
        logical_date: value.logical_date,
        start_date: value.start_date,
        end_date: value.end_date,
        duration: value.duration,
        state: value.state.map(|s| TaskInstanceState::from(s.as_str())),
        try_number: value.try_number,
        max_tries: value.max_tries,
        map_index: value.map_index,
        hostname: value.hostname,
        unixname: value.unixname,
        pool: value.pool,
        pool_slots: value.pool_slots,
        queue: value.queue,
        priority_weight: value.priority_weight,
        operator: value.operator,
        queued_when: value.queued_when,
        pid: value.pid,
        note: value.note,
    }
}

fn v2_task_instance_list_to_list(
    value: flowrs_airflow::client::v2::model::taskinstance::TaskInstanceList,
) -> TaskInstanceList {
    TaskInstanceList {
        task_instances: value
            .task_instances
            .into_iter()
            .map(v2_task_instance_to_domain)
            .collect(),
        total_entries: value.total_entries,
    }
}

fn v2_task_instance_try_to_gantt(
    value: flowrs_airflow::client::v2::model::taskinstance::TaskInstanceTryResponse,
) -> TaskTryGantt {
    TaskTryGantt {
        try_number: value.try_number,
        start_date: value.start_date,
        end_date: value.end_date,
        state: value.state.map(|s| TaskInstanceState::from(s.as_str())),
    }
}

fn v2_task_to_task(value: flowrs_airflow::client::v2::model::task::TaskResponse) -> Task {
    Task {
        task_id: value.task_id,
        downstream_task_ids: value.downstream_task_ids,
    }
}

fn v2_task_collection_to_list(
    value: flowrs_airflow::client::v2::model::task::TaskCollectionResponse,
) -> TaskList {
    TaskList {
        tasks: value.tasks.into_iter().map(v2_task_to_task).collect(),
    }
}

fn v2_log_to_log(value: flowrs_airflow::client::v2::model::log::Log) -> Log {
    Log {
        continuation_token: value.continuation_token,
        content: value.content.to_string(),
    }
}

fn v2_dagstats_to_response(
    value: flowrs_airflow::client::v2::model::dagstats::DagStatsResponse,
) -> DagStatsResponse {
    DagStatsResponse {
        dags: value
            .dags
            .into_iter()
            .map(|ds| DagStatistics {
                dag_id: ds.dag_id,
                stats: ds
                    .stats
                    .into_iter()
                    .map(|s| DagStatistic {
                        state: DagRunState::from(s.state.as_str()),
                        count: s.count,
                    })
                    .collect(),
            })
            .collect(),
        total_entries: value.total_entries,
    }
}

// =============================================================================
// URL building helpers (moved from v1/mod.rs and v2/mod.rs)
// =============================================================================

fn build_v1_open_url(endpoint: &str, item: &OpenItem) -> Result<String> {
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
            base_url = base_url.join(&format!("dags/{dag_id}/grid"))?;
            base_url.set_query(Some(&format!(
                "dag_run_id={escaped_dag_run_id}&task_id={task_id}"
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
            base_url = base_url.join(&format!("dags/{dag_id}/grid"))?;
            base_url.set_query(Some(&format!(
                "dag_run_id={escaped_dag_run_id}&task_id={task_id}&tab=logs"
            )));
        }
    }

    Ok(base_url.to_string())
}

fn build_v2_open_url(endpoint: &str, item: &OpenItem) -> Result<String> {
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
