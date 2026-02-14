use std::sync::LazyLock;

use crate::airflow::config::FlowrsConfig;
use crate::airflow::model::common::{DagId, DagRunId, TaskId};
use crate::app::environment_state::EnvironmentStateContainer;
use crate::app::model::dagruns::DagRunModel;
use crate::app::model::dags::DagModel;
use crate::app::model::popup::warning::WarningPopup;
use throbber_widgets_tui::ThrobberState;
use time::format_description::BorrowedFormatItem;

use super::model::{config::ConfigModel, logs::LogModel, taskinstances::TaskInstanceModel};
use super::worker::WorkerMessage;

/// Cached date format for breadcrumb display (YYYY-MM-DD)
static BREADCRUMB_DATE_FORMAT: LazyLock<Vec<BorrowedFormatItem<'static>>> = LazyLock::new(|| {
    time::format_description::parse("[year]-[month]-[day]").expect("Invalid date format")
});

/// Centralized navigation context — the single source of truth for what the
/// user is currently looking at. Replaces the redundant per-panel context IDs
/// that were previously stored on `DagRunModel`, `TaskInstanceModel`, and `LogModel`.
#[derive(Clone, Default, Debug)]
pub struct NavigationContext {
    pub environment: Option<String>,
    pub dag_id: Option<DagId>,
    pub dag_run_id: Option<DagRunId>,
    pub task_id: Option<TaskId>,
    pub task_try: Option<u16>,
}

pub struct App {
    pub config: FlowrsConfig,
    pub environment_state: EnvironmentStateContainer,
    pub nav_context: NavigationContext,
    pub dags: DagModel,
    pub configs: ConfigModel,
    pub dagruns: DagRunModel,
    pub task_instances: TaskInstanceModel,
    pub logs: LogModel,
    pub ticks: u32,
    pub active_panel: Panel,
    pub loading: bool,
    pub startup: bool,
    pub throbber_state: ThrobberState,
    /// Global warning popup shown on startup (e.g., legacy config conflict)
    pub warning_popup: Option<WarningPopup>,
    /// Whether the terminal window has focus (used to pause refreshes when unfocused)
    pub focused: bool,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Panel {
    Config,
    Dag,
    DAGRun,
    TaskInstance,
    Logs,
}

impl App {
    #[allow(dead_code)]
    pub fn new(config: FlowrsConfig) -> Self {
        Self::new_with_errors_and_warnings(config, vec![], vec![])
    }

    pub fn new_with_errors_and_warnings(
        config: FlowrsConfig,
        errors: Vec<String>,
        warnings: Vec<String>,
    ) -> Self {
        let servers = config.servers.clone();
        let has_active_server = config
            .active_server
            .as_ref()
            .is_some_and(|name| servers.iter().any(|server| &server.name == name));

        let warning_popup = if warnings.is_empty() {
            None
        } else {
            Some(WarningPopup::new(warnings))
        };

        let nav_context = NavigationContext {
            environment: config.active_server.clone(),
            ..Default::default()
        };

        Self {
            config,
            environment_state: EnvironmentStateContainer::new(),
            nav_context,
            dags: DagModel::new(),
            configs: ConfigModel::new_with_errors(&servers, errors),
            dagruns: DagRunModel::new(),
            task_instances: TaskInstanceModel::new(),
            logs: LogModel::new(),
            active_panel: if has_active_server {
                Panel::Dag
            } else {
                Panel::Config
            },
            ticks: 0,
            loading: true,
            startup: true,
            throbber_state: ThrobberState::default(),
            warning_popup,
            focused: true,
        }
    }

    pub const fn next_panel(&mut self) {
        match self.active_panel {
            Panel::Config => self.active_panel = Panel::Dag,
            Panel::Dag => self.active_panel = Panel::DAGRun,
            Panel::DAGRun => self.active_panel = Panel::TaskInstance,
            Panel::TaskInstance => self.active_panel = Panel::Logs,
            Panel::Logs => (),
        }
    }

    pub const fn previous_panel(&mut self) {
        match self.active_panel {
            Panel::Config => (),
            Panel::Dag => self.active_panel = Panel::Config,
            Panel::DAGRun => self.active_panel = Panel::Dag,
            Panel::TaskInstance => self.active_panel = Panel::DAGRun,
            Panel::Logs => self.active_panel = Panel::TaskInstance,
        }
    }

    pub fn clear_state(&mut self) {
        self.ticks = 0;
        self.loading = true;
        // Clear navigation context below the environment level
        self.nav_context.dag_id = None;
        self.nav_context.dag_run_id = None;
        self.nav_context.task_id = None;
        self.nav_context.task_try = None;
        // Clear view models but not environment_state
        self.dags.table.all.clear();
        self.dagruns.table.all.clear();
        self.task_instances.table.all.clear();
        self.logs.all.clear();
    }

    /// Generate breadcrumb string showing navigation context.
    /// Returns a path like `"dev > dag_example > manual_2025... > task_1"`
    pub fn breadcrumb(&self) -> Option<String> {
        let mut parts: Vec<String> = Vec::new();

        // Always start with the active environment name
        let env_name = self.nav_context.environment.as_ref()?;
        parts.push(env_name.clone());

        // Add DAG if we're past the Config panel
        if self.active_panel != Panel::Config && self.active_panel != Panel::Dag {
            if let Some(dag_id) = &self.nav_context.dag_id {
                let truncated = Self::truncate_breadcrumb_part(dag_id, 25);
                parts.push(truncated);
            }
        }

        // Add DAG Run logical date if we're at TaskInstance or Logs panel
        if self.active_panel == Panel::TaskInstance || self.active_panel == Panel::Logs {
            if let Some(task_instance) = self.task_instances.table.filtered.items.first() {
                if let Some(logical_date) = task_instance.logical_date {
                    let formatted = logical_date
                        .format(&BREADCRUMB_DATE_FORMAT)
                        .unwrap_or_else(|_| "unknown".to_string());
                    parts.push(formatted);
                }
            } else if let Some(dag_run_id) = &self.nav_context.dag_run_id {
                let truncated = Self::truncate_breadcrumb_part(dag_run_id, 20);
                parts.push(truncated);
            }
        }

        // Add Task if we're at Logs panel
        if self.active_panel == Panel::Logs {
            if let Some(task_id) = &self.nav_context.task_id {
                let truncated = Self::truncate_breadcrumb_part(task_id, 20);
                parts.push(truncated);
            }
        }

        if parts.len() > 1 || self.active_panel != Panel::Config {
            Some(parts.join(" > "))
        } else if parts.len() == 1 {
            Some(parts[0].clone())
        } else {
            None
        }
    }

    /// Truncate a string for breadcrumb display, adding "..." if needed.
    /// Uses char boundaries to avoid panicking on multi-byte UTF-8 characters.
    fn truncate_breadcrumb_part(s: &str, max_chars: usize) -> String {
        let char_count = s.chars().count();
        if char_count <= max_chars {
            s.to_string()
        } else {
            let truncate_at = max_chars.saturating_sub(3);
            let byte_index = s
                .char_indices()
                .nth(truncate_at)
                .map_or(s.len(), |(idx, _)| idx);
            format!("{}...", &s[..byte_index])
        }
    }

    /// Update the centralized navigation context from a `WorkerMessage`.
    /// This is the single place where navigation state changes in response
    /// to panel-emitted messages.
    pub fn set_context_from_message(&mut self, message: &WorkerMessage) {
        match message {
            WorkerMessage::UpdateDagRuns { dag_id } => {
                self.nav_context.dag_id = Some(dag_id.clone());
                // Clear deeper context when navigating to a new DAG
                self.nav_context.dag_run_id = None;
                self.nav_context.task_id = None;
                self.nav_context.task_try = None;
            }
            WorkerMessage::UpdateTaskInstances { dag_id, dag_run_id } => {
                self.nav_context.dag_id = Some(dag_id.clone());
                self.nav_context.dag_run_id = Some(dag_run_id.clone());
                // Clear deeper context
                self.nav_context.task_id = None;
                self.nav_context.task_try = None;
            }
            WorkerMessage::UpdateTaskLogs {
                dag_id,
                dag_run_id,
                task_id,
                task_try,
            } => {
                let is_new_context = self.nav_context.dag_id.as_ref() != Some(dag_id)
                    || self.nav_context.dag_run_id.as_ref() != Some(dag_run_id)
                    || self.nav_context.task_id.as_ref() != Some(task_id)
                    || self.nav_context.task_try.as_ref() != Some(task_try);
                self.nav_context.dag_id = Some(dag_id.clone());
                self.nav_context.dag_run_id = Some(dag_run_id.clone());
                self.nav_context.task_id = Some(task_id.clone());
                self.nav_context.task_try = Some(*task_try);
                if is_new_context {
                    self.logs.current = 0;
                    self.logs.follow_mode = true;
                }
            }
            WorkerMessage::UpdateTasks { dag_id } => {
                if self.nav_context.dag_id.as_ref() != Some(dag_id) {
                    self.task_instances.task_graph = None;
                }
            }
            _ => {}
        }
    }

    /// Sync a specific panel's data from `environment_state`.
    pub fn sync_panel(&mut self, panel: &Panel) {
        match panel {
            Panel::Dag => {
                self.dags.table.all = self.environment_state.get_active_dags();
                self.dags.dag_stats = self.environment_state.get_active_dag_stats();
                let dag_ids: Vec<String> = self
                    .dags
                    .table
                    .all
                    .iter()
                    .map(|d| d.dag_id.to_string())
                    .collect();
                self.dags.table.filter.set_primary_values("dag_id", dag_ids);
                self.dags.table.apply_filter();
            }
            Panel::DAGRun => {
                if let Some(dag_id) = &self.nav_context.dag_id {
                    self.dagruns.table.all = self.environment_state.get_active_dag_runs(dag_id);
                    let dag_run_ids: Vec<String> = self
                        .dagruns
                        .table
                        .all
                        .iter()
                        .map(|dr| dr.dag_run_id.to_string())
                        .collect();
                    self.dagruns
                        .table
                        .filter
                        .set_primary_values("dag_run_id", dag_run_ids);
                    self.dagruns.table.apply_filter();
                    self.dagruns.sort_dag_runs();
                } else {
                    self.dagruns.table.all.clear();
                }
            }
            Panel::TaskInstance => {
                if let (Some(dag_id), Some(dag_run_id)) =
                    (&self.nav_context.dag_id, &self.nav_context.dag_run_id)
                {
                    self.task_instances.table.all = self
                        .environment_state
                        .get_active_task_instances(dag_id, dag_run_id);
                    self.task_instances.sort_task_instances();
                    let task_ids: Vec<String> = self
                        .task_instances
                        .table
                        .all
                        .iter()
                        .map(|ti| ti.task_id.to_string())
                        .collect();
                    self.task_instances
                        .table
                        .filter
                        .set_primary_values("task_id", task_ids);
                    self.task_instances.table.apply_filter();
                } else {
                    self.task_instances.table.all.clear();
                }
            }
            Panel::Logs => {
                if let (Some(dag_id), Some(dag_run_id), Some(task_id)) = (
                    &self.nav_context.dag_id,
                    &self.nav_context.dag_run_id,
                    &self.nav_context.task_id,
                ) {
                    self.logs.update_logs(
                        self.environment_state
                            .get_active_task_logs(dag_id, dag_run_id, task_id),
                    );
                } else {
                    self.logs.all.clear();
                }
            }
            Panel::Config => {
                let config_names: Vec<String> = self
                    .configs
                    .table
                    .all
                    .iter()
                    .map(|c| c.name.clone())
                    .collect();
                self.configs
                    .table
                    .filter
                    .set_primary_values("name", config_names);
            }
        }
    }
}
