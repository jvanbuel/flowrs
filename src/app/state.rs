use std::sync::LazyLock;

use crate::airflow::config::FlowrsConfig;
use crate::app::environment_state::EnvironmentStateContainer;
use crate::app::model::dagruns::DagRunModel;
use crate::app::model::dags::DagModel;
use crate::app::model::popup::warning::WarningPopup;
use throbber_widgets_tui::ThrobberState;
use time::format_description::BorrowedFormatItem;

use super::model::{config::ConfigModel, logs::LogModel, taskinstances::TaskInstanceModel};

/// Cached date format for breadcrumb display (YYYY-MM-DD)
static BREADCRUMB_DATE_FORMAT: LazyLock<Vec<BorrowedFormatItem<'static>>> = LazyLock::new(|| {
    time::format_description::parse("[year]-[month]-[day]").expect("Invalid date format")
});

pub struct App {
    pub config: FlowrsConfig,
    pub environment_state: EnvironmentStateContainer,
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

#[derive(Clone, PartialEq)]
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
        let servers = &config.clone().servers.unwrap_or_default();
        let active_server = config
            .active_server
            .as_ref()
            .and_then(|name| servers.iter().find(|server| &server.name == name));

        let warning_popup = if warnings.is_empty() {
            None
        } else {
            Some(WarningPopup::new(warnings))
        };

        App {
            config,
            environment_state: EnvironmentStateContainer::new(),
            dags: DagModel::new(),
            configs: ConfigModel::new_with_errors(servers.clone(), errors),
            dagruns: DagRunModel::new(),
            task_instances: TaskInstanceModel::new(),
            logs: LogModel::new(),
            active_panel: match active_server {
                Some(_) => Panel::Dag,
                None => Panel::Config,
            },
            ticks: 0,
            loading: true,
            startup: true,
            throbber_state: ThrobberState::default(),
            warning_popup,
            focused: true,
        }
    }

    pub fn next_panel(&mut self) {
        match self.active_panel {
            Panel::Config => self.active_panel = Panel::Dag,
            Panel::Dag => self.active_panel = Panel::DAGRun,
            Panel::DAGRun => self.active_panel = Panel::TaskInstance,
            Panel::TaskInstance => self.active_panel = Panel::Logs,
            Panel::Logs => (),
        }
    }

    pub fn previous_panel(&mut self) {
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
        // Clear view models but not environment_state
        // This clears UI state (filters, selections) but data persists in environment_state
        self.dags.all.clear();
        self.dagruns.all.clear();
        self.task_instances.all.clear();
        self.logs.all.clear();
    }

    /// Generate breadcrumb string showing navigation context.
    /// Returns a path like `"dev > dag_example > manual_2025... > task_1"`
    pub fn breadcrumb(&self) -> Option<String> {
        let mut parts: Vec<String> = Vec::new();

        // Always start with the active environment (config) name if set
        let env_name = self.config.active_server.as_ref()?;
        parts.push(env_name.clone());

        // Add DAG if we're past the Config panel
        if self.active_panel != Panel::Config && self.active_panel != Panel::Dag {
            if let Some(dag_id) = &self.dagruns.dag_id {
                // Truncate long dag_id
                let truncated = Self::truncate_breadcrumb_part(dag_id, 25);
                parts.push(truncated);
            }
        }

        // Add DAG Run logical date if we're at TaskInstance or Logs panel
        if self.active_panel == Panel::TaskInstance || self.active_panel == Panel::Logs {
            // Get logical_date from the first task instance (they all share the same dag run)
            if let Some(task_instance) = self.task_instances.filtered.items.first() {
                if let Some(logical_date) = task_instance.logical_date {
                    let formatted = logical_date
                        .format(&BREADCRUMB_DATE_FORMAT)
                        .unwrap_or_else(|_| "unknown".to_string());
                    parts.push(formatted);
                }
            } else if let Some(dag_run_id) = &self.task_instances.dag_run_id {
                // Fallback to dag_run_id if no task instances loaded yet
                let truncated = Self::truncate_breadcrumb_part(dag_run_id, 20);
                parts.push(truncated);
            }
        }

        // Add Task if we're at Logs panel
        if self.active_panel == Panel::Logs {
            if let Some(task_id) = &self.logs.task_id {
                // Truncate long task_id
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
            // Find the byte index at the char boundary for truncation
            let truncate_at = max_chars.saturating_sub(3);
            let byte_index = s
                .char_indices()
                .nth(truncate_at)
                .map_or(s.len(), |(idx, _)| idx);
            format!("{}...", &s[..byte_index])
        }
    }

    /// Sync panel data from `environment_state`
    /// This should be called when switching panels or environments
    pub fn sync_panel_data(&mut self) {
        match self.active_panel {
            Panel::Dag => {
                self.dags.all = self.environment_state.get_active_dags();
                self.dags.dag_stats = self.environment_state.get_active_dag_stats();
                self.dags.filter_dags();
            }
            Panel::DAGRun => {
                if let Some(dag_id) = &self.dagruns.dag_id {
                    self.dagruns.all = self.environment_state.get_active_dag_runs(dag_id);
                    self.dagruns.filter_dag_runs();
                } else {
                    self.dagruns.all.clear();
                }
            }
            Panel::TaskInstance => {
                if let (Some(dag_id), Some(dag_run_id)) =
                    (&self.task_instances.dag_id, &self.task_instances.dag_run_id)
                {
                    self.task_instances.all = self
                        .environment_state
                        .get_active_task_instances(dag_id, dag_run_id);
                    self.task_instances.filter_task_instances();
                } else {
                    self.task_instances.all.clear();
                }
            }
            Panel::Logs => {
                if let (Some(dag_id), Some(dag_run_id), Some(task_id)) =
                    (&self.logs.dag_id, &self.logs.dag_run_id, &self.logs.task_id)
                {
                    self.logs.all = self
                        .environment_state
                        .get_active_task_logs(dag_id, dag_run_id, task_id);
                } else {
                    self.logs.all.clear();
                }
            }
            Panel::Config => {
                // Config panel doesn't need syncing
            }
        }
    }
}
