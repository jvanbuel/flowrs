use crate::airflow::model::common::{DagId, DagRunId, TaskId};

/// Centralized navigation context — the single source of truth for what the
/// user is currently looking at. Encoded as an enum to enforce the strict
/// hierarchy: environment > dag > `dag_run` > task. It is impossible to have,
/// e.g., a `task_id` without a `dag_id`.
#[derive(Clone, Debug, Default)]
pub enum NavigationContext {
    #[default]
    None,
    Environment {
        environment: String,
    },
    Dag {
        environment: String,
        dag_id: DagId,
    },
    DagRun {
        environment: String,
        dag_id: DagId,
        dag_run_id: DagRunId,
    },
    Task {
        environment: String,
        dag_id: DagId,
        dag_run_id: DagRunId,
        task_id: TaskId,
        task_try: u32,
    },
}

impl NavigationContext {
    pub fn environment(&self) -> Option<&String> {
        match self {
            NavigationContext::None => Option::None,
            NavigationContext::Environment { environment, .. }
            | NavigationContext::Dag { environment, .. }
            | NavigationContext::DagRun { environment, .. }
            | NavigationContext::Task { environment, .. } => Some(environment),
        }
    }

    pub fn dag_id(&self) -> Option<&DagId> {
        match self {
            NavigationContext::None | NavigationContext::Environment { .. } => Option::None,
            NavigationContext::Dag { dag_id, .. }
            | NavigationContext::DagRun { dag_id, .. }
            | NavigationContext::Task { dag_id, .. } => Some(dag_id),
        }
    }

    pub fn dag_run_id(&self) -> Option<&DagRunId> {
        match self {
            NavigationContext::None
            | NavigationContext::Environment { .. }
            | NavigationContext::Dag { .. } => Option::None,
            NavigationContext::DagRun { dag_run_id, .. }
            | NavigationContext::Task { dag_run_id, .. } => Some(dag_run_id),
        }
    }

    pub fn task_id(&self) -> Option<&TaskId> {
        match self {
            NavigationContext::Task { task_id, .. } => Some(task_id),
            _ => Option::None,
        }
    }

    pub fn task_try(&self) -> Option<u32> {
        match self {
            NavigationContext::Task { task_try, .. } => Some(*task_try),
            _ => Option::None,
        }
    }

    /// Reset to environment-only level, discarding any deeper context.
    pub fn reset_to_environment(&mut self) {
        *self = match self.environment() {
            Some(env) => NavigationContext::Environment {
                environment: env.clone(),
            },
            Option::None => NavigationContext::None,
        };
    }
}
