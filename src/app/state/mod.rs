mod breadcrumb;
mod context;
mod navigation;
mod sync;

pub mod environment_state;

pub use navigation::NavigationContext;

use crate::app::model::dagruns::DagRunModel;
use crate::app::model::dags::DagModel;
use crate::app::model::popup::warning::WarningPopup;
use environment_state::EnvironmentStateContainer;
use flowrs_config::FlowrsConfig;
use throbber_widgets_tui::ThrobberState;

use super::model::{config::ConfigModel, logs::LogModel, taskinstances::TaskInstanceModel};

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

        let nav_context = match config.active_server.clone() {
            Some(env) => NavigationContext::Environment { environment: env },
            None => NavigationContext::None,
        };

        let poll_tick_multiplier = config.poll_tick_multiplier();

        Self {
            config,
            environment_state: EnvironmentStateContainer::new(),
            nav_context,
            dags: DagModel::new(poll_tick_multiplier),
            configs: ConfigModel::new_with_errors(&servers, errors),
            dagruns: DagRunModel::new(poll_tick_multiplier),
            task_instances: TaskInstanceModel::new(poll_tick_multiplier),
            logs: LogModel::new(poll_tick_multiplier),
            active_panel: if has_active_server {
                Panel::Dag
            } else {
                Panel::Config
            },
            ticks: 0,
            loading: true,
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
        self.loading = true;
        self.nav_context.reset_to_environment();
        self.dags.table.all.clear();
        self.dagruns.table.all.clear();
        self.task_instances.table.all.clear();
        self.logs.all.clear();
    }
}
