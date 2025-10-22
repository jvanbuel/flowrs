use crate::airflow::config::FlowrsConfig;
use crate::app::environment_state::EnvironmentStateContainer;
use crate::app::model::dagruns::DagRunModel;
use crate::app::model::dags::DagModel;
use throbber_widgets_tui::ThrobberState;

use super::model::{config::ConfigModel, logs::LogModel, taskinstances::TaskInstanceModel};

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
        Self::new_with_errors(config, vec![])
    }

    pub fn new_with_errors(config: FlowrsConfig, errors: Vec<String>) -> Self {
        let servers = &config.clone().servers.unwrap_or_default();
        let active_server = if let Some(active_server) = &config.active_server {
            servers.iter().find(|server| server.name == *active_server)
        } else {
            None
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

    /// Sync panel data from `environment_state`
    /// This should be called when switching panels or environments
    pub fn sync_panel_data(&mut self) {
        match self.active_panel {
            Panel::Dag => {
                self.dags.all = self.environment_state.get_active_dags();
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
