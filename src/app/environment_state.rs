use std::collections::HashMap;
use std::sync::Arc;

use crate::airflow::{
    model::common::{Dag, DagRun, DagStatistic, Log, TaskInstance},
    traits::AirflowClient as AirflowClientTrait,
};

/// Key identifying an environment (Airflow server configuration)
pub type EnvironmentKey = String;
pub type DagId = String;
pub type DagRunId = String;
pub type TaskId = String;

/// Flat, request-keyed cache for a single Airflow environment.
///
/// Each collection is keyed by the parameters of the API request that produced
/// it. This replaces the previous nested `DagData → DagRunData → TaskInstanceData`
/// hierarchy with direct lookups, and makes stale-entry eviction trivial
/// (just replace the whole Vec).
#[derive(Clone)]
pub struct EnvironmentData {
    pub client: Arc<dyn AirflowClientTrait>,

    /// Result of `list_dags()` — sorted alphabetically by `dag_id` on write.
    pub dags: Vec<Dag>,

    /// Result of `get_dag_stats(dag_ids)` — keyed per DAG.
    pub dag_stats: HashMap<DagId, Vec<DagStatistic>>,

    /// Result of `list_dagruns(dag_id)` — keyed by `dag_id`.
    pub dag_runs: HashMap<DagId, Vec<DagRun>>,

    /// Result of `list_task_instances(dag_id, dag_run_id)` — keyed by (`dag_id`, `dag_run_id`).
    pub task_instances: HashMap<DagId, HashMap<DagRunId, Vec<TaskInstance>>>,

    /// Result of `get_task_logs(dag_id, dag_run_id, task_id, try)` — keyed by (`dag_id`, `dag_run_id`, `task_id`).
    pub task_logs: HashMap<DagId, HashMap<DagRunId, HashMap<TaskId, Vec<Log>>>>,
}

impl EnvironmentData {
    pub fn new(client: Arc<dyn AirflowClientTrait>) -> Self {
        Self {
            client,
            dags: Vec::new(),
            dag_stats: HashMap::new(),
            dag_runs: HashMap::new(),
            task_instances: HashMap::new(),
            task_logs: HashMap::new(),
        }
    }

    // ── Write methods (called by workers after API responses) ────────

    /// Replace the full DAG list (evicts deleted DAGs).
    pub fn replace_dags(&mut self, mut dags: Vec<Dag>) {
        dags.sort_by(|a, b| a.dag_id.cmp(&b.dag_id));
        self.dags = dags;
    }

    /// Replace stats for a single DAG.
    pub fn update_dag_stats(&mut self, dag_id: &str, stats: Vec<DagStatistic>) {
        self.dag_stats.insert(dag_id.to_string(), stats);
    }

    /// Replace all DAG runs for a DAG (evicts deleted runs).
    pub fn replace_dag_runs(&mut self, dag_id: &str, dag_runs: Vec<DagRun>) {
        self.dag_runs.insert(dag_id.to_string(), dag_runs);
    }

    /// Replace all task instances for a DAG run (evicts deleted instances).
    pub fn replace_task_instances(
        &mut self,
        dag_id: &str,
        dag_run_id: &str,
        task_instances: Vec<TaskInstance>,
    ) {
        self.task_instances
            .entry(dag_id.to_string())
            .or_default()
            .insert(dag_run_id.to_string(), task_instances);
    }

    /// Replace logs for a specific task instance.
    pub fn add_task_logs(
        &mut self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
        logs: Vec<Log>,
    ) {
        self.task_logs
            .entry(dag_id.to_string())
            .or_default()
            .entry(dag_run_id.to_string())
            .or_default()
            .insert(task_id.to_string(), logs);
    }
}

/// Container for all environment states
#[derive(Clone)]
pub struct EnvironmentStateContainer {
    pub environments: HashMap<EnvironmentKey, EnvironmentData>,
    pub active_environment: Option<EnvironmentKey>,
}

impl EnvironmentStateContainer {
    pub fn new() -> Self {
        Self {
            environments: HashMap::new(),
            active_environment: None,
        }
    }

    pub fn add_environment(&mut self, key: EnvironmentKey, data: EnvironmentData) {
        self.environments.insert(key, data);
    }

    pub fn get_active_environment(&self) -> Option<&EnvironmentData> {
        self.active_environment
            .as_ref()
            .and_then(|key| self.environments.get(key))
    }

    pub fn get_active_environment_mut(&mut self) -> Option<&mut EnvironmentData> {
        self.active_environment
            .as_ref()
            .and_then(|key| self.environments.get_mut(key))
    }

    pub fn set_active_environment(&mut self, key: EnvironmentKey) {
        if self.environments.contains_key(&key) {
            self.active_environment = Some(key);
        }
    }

    pub fn get_active_client(&self) -> Option<Arc<dyn AirflowClientTrait>> {
        self.get_active_environment().map(|env| env.client.clone())
    }

    // ── Read methods (called by sync_panel and workers) ─────────────

    /// Get all DAGs for the active environment (already sorted).
    pub fn get_active_dags(&self) -> Vec<Dag> {
        self.get_active_environment()
            .map(|env| env.dags.clone())
            .unwrap_or_default()
    }

    /// Get a specific DAG by ID from the active environment.
    pub fn get_active_dag(&self, dag_id: &str) -> Option<Dag> {
        self.get_active_environment()
            .and_then(|env| env.dags.iter().find(|d| d.dag_id == dag_id).cloned())
    }

    /// Get all DAG statistics for the active environment.
    pub fn get_active_dag_stats(&self) -> HashMap<String, Vec<DagStatistic>> {
        self.get_active_environment()
            .map(|env| env.dag_stats.clone())
            .unwrap_or_default()
    }

    /// Get all DAG runs for a specific DAG in the active environment.
    pub fn get_active_dag_runs(&self, dag_id: &str) -> Vec<DagRun> {
        self.get_active_environment()
            .and_then(|env| env.dag_runs.get(dag_id))
            .cloned()
            .unwrap_or_default()
    }

    /// Get all task instances for a specific DAG run in the active environment.
    pub fn get_active_task_instances(&self, dag_id: &str, dag_run_id: &str) -> Vec<TaskInstance> {
        self.get_active_environment()
            .and_then(|env| env.task_instances.get(dag_id))
            .and_then(|runs| runs.get(dag_run_id))
            .cloned()
            .unwrap_or_default()
    }

    /// Get logs for a specific task instance in the active environment.
    pub fn get_active_task_logs(
        &self,
        dag_id: &str,
        dag_run_id: &str,
        task_id: &str,
    ) -> Vec<Log> {
        self.get_active_environment()
            .and_then(|env| env.task_logs.get(dag_id))
            .and_then(|runs| runs.get(dag_run_id))
            .and_then(|tasks| tasks.get(task_id))
            .cloned()
            .unwrap_or_default()
    }
}

impl Default for EnvironmentStateContainer {
    fn default() -> Self {
        Self::new()
    }
}
