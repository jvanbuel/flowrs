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

/// State for a specific task instance's logs
#[derive(Debug, Clone)]
pub struct TaskInstanceData {
    pub task_instance: TaskInstance,
    pub logs: Vec<Log>,
}

impl TaskInstanceData {
    pub const fn new(task_instance: TaskInstance) -> Self {
        Self {
            task_instance,
            logs: Vec::new(),
        }
    }
}

/// State for a specific DAG run
#[derive(Debug, Clone)]
pub struct DagRunData {
    pub dag_run: DagRun,
    pub task_instances: HashMap<TaskId, TaskInstanceData>,
}

impl DagRunData {
    pub fn new(dag_run: DagRun) -> Self {
        Self {
            dag_run,
            task_instances: HashMap::new(),
        }
    }
    pub fn get_task_instance(&self, task_id: &str) -> Option<&TaskInstanceData> {
        self.task_instances.get(task_id)
    }
}

/// State for a specific DAG
#[derive(Debug, Clone)]
pub struct DagData {
    pub dag: Dag,
    pub dag_runs: HashMap<DagRunId, DagRunData>,
    pub stats: Vec<DagStatistic>,
}

impl DagData {
    pub fn new(dag: Dag) -> Self {
        Self {
            dag,
            dag_runs: HashMap::new(),
            stats: Vec::new(),
        }
    }

    pub fn get_dag_run(&self, dag_run_id: &str) -> Option<&DagRunData> {
        self.dag_runs.get(dag_run_id)
    }
}

/// State for a specific environment (Airflow server)
#[derive(Clone)]
pub struct EnvironmentData {
    pub client: Arc<dyn AirflowClientTrait>,
    pub dags: HashMap<DagId, DagData>,
}

impl EnvironmentData {
    pub fn new(client: Arc<dyn AirflowClientTrait>) -> Self {
        Self {
            client,
            dags: HashMap::new(),
        }
    }

    pub fn get_dag(&self, dag_id: &str) -> Option<&DagData> {
        self.dags.get(dag_id)
    }

    /// Update or create a DAG in the environment
    pub fn upsert_dag(&mut self, dag: Dag) {
        let dag_id = dag.dag_id.clone();
        if let Some(existing_dag_data) = self.dags.get_mut(&dag_id) {
            existing_dag_data.dag = dag;
        } else {
            self.dags.insert(dag_id, DagData::new(dag));
        }
    }

    /// Update or create a DAG run in the environment
    pub fn upsert_dag_run(&mut self, dag_run: DagRun) {
        let dag_id = dag_run.dag_id.clone();
        let dag_run_id = dag_run.dag_run_id.clone();

        if let Some(dag_data) = self.dags.get_mut(&dag_id) {
            if let Some(existing_run) = dag_data.dag_runs.get_mut(&dag_run_id) {
                existing_run.dag_run = dag_run;
            } else {
                dag_data
                    .dag_runs
                    .insert(dag_run_id, DagRunData::new(dag_run));
            }
        }
    }

    /// Update or create a task instance in the environment
    pub fn upsert_task_instance(&mut self, task_instance: TaskInstance) {
        let dag_id = task_instance.dag_id.clone();
        let dag_run_id = task_instance.dag_run_id.clone();
        let task_id = task_instance.task_id.clone();

        if let Some(dag_data) = self.dags.get_mut(&dag_id) {
            if let Some(dag_run_data) = dag_data.dag_runs.get_mut(&dag_run_id) {
                if let Some(existing_task) = dag_run_data.task_instances.get_mut(&task_id) {
                    existing_task.task_instance = task_instance;
                } else {
                    dag_run_data
                        .task_instances
                        .insert(task_id, TaskInstanceData::new(task_instance));
                }
            }
        }
    }

    /// Add logs to a task instance
    pub fn add_task_logs(&mut self, dag_id: &str, dag_run_id: &str, task_id: &str, logs: Vec<Log>) {
        if let Some(dag_data) = self.dags.get_mut(dag_id) {
            if let Some(dag_run_data) = dag_data.dag_runs.get_mut(dag_run_id) {
                if let Some(task_data) = dag_run_data.task_instances.get_mut(task_id) {
                    task_data.logs = logs;
                }
            }
        }
    }

    /// Update statistics for a specific DAG
    pub fn update_dag_stats(&mut self, dag_id: &str, stats: Vec<DagStatistic>) {
        if let Some(dag_data) = self.dags.get_mut(dag_id) {
            dag_data.stats = stats;
        }
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

    /// Get all DAGs for the active environment, sorted alphabetically by `dag_id`
    pub fn get_active_dags(&self) -> Vec<Dag> {
        self.get_active_environment()
            .map(|env| {
                let mut dags: Vec<Dag> = env
                    .dags
                    .values()
                    .map(|dag_data| dag_data.dag.clone())
                    .collect();
                dags.sort_by(|a, b| a.dag_id.cmp(&b.dag_id));
                dags
            })
            .unwrap_or_default()
    }

    /// Get all DAG runs for a specific DAG in the active environment
    pub fn get_active_dag_runs(&self, dag_id: &str) -> Vec<DagRun> {
        self.get_active_environment()
            .and_then(|env| env.get_dag(dag_id))
            .map(|dag_data| {
                dag_data
                    .dag_runs
                    .values()
                    .map(|run_data| run_data.dag_run.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all task instances for a specific DAG run in the active environment
    pub fn get_active_task_instances(&self, dag_id: &str, dag_run_id: &str) -> Vec<TaskInstance> {
        self.get_active_environment()
            .and_then(|env| env.get_dag(dag_id))
            .and_then(|dag_data| dag_data.get_dag_run(dag_run_id))
            .map(|run_data| {
                run_data
                    .task_instances
                    .values()
                    .map(|task_data| task_data.task_instance.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get logs for a specific task instance in the active environment
    pub fn get_active_task_logs(&self, dag_id: &str, dag_run_id: &str, task_id: &str) -> Vec<Log> {
        self.get_active_environment()
            .and_then(|env| env.get_dag(dag_id))
            .and_then(|dag_data| dag_data.get_dag_run(dag_run_id))
            .and_then(|run_data| run_data.get_task_instance(task_id))
            .map(|task_data| task_data.logs.clone())
            .unwrap_or_default()
    }

    /// Get a specific DAG by ID from the active environment
    pub fn get_active_dag(&self, dag_id: &str) -> Option<Dag> {
        self.get_active_environment()
            .and_then(|env| env.get_dag(dag_id))
            .map(|dag_data| dag_data.dag.clone())
    }

    /// Get all DAG statistics for the active environment as a `HashMap`
    pub fn get_active_dag_stats(&self) -> HashMap<String, Vec<DagStatistic>> {
        self.get_active_environment()
            .map(|env| {
                env.dags
                    .iter()
                    .filter(|(_, dag_data)| !dag_data.stats.is_empty())
                    .map(|(dag_id, dag_data)| (dag_id.clone(), dag_data.stats.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Default for EnvironmentStateContainer {
    fn default() -> Self {
        Self::new()
    }
}
