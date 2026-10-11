mod breadcrumb;
mod sync;

pub mod environment_state;

use crate::app::events::custom::FlowrsEvent;
use crate::app::model::dagruns::DagRunModel;
use crate::app::model::dags::DagModel;
use crate::app::model::popup::error::ErrorPopup;
use crate::app::model::popup::warning::WarningPopup;
use crate::app::model::Model;
use crate::app::navigation::{Level, Move, Navigation, Selection};
use crate::app::worker::WorkerMessage;
use environment_state::EnvironmentStateContainer;
use flowrs_config::FlowrsConfig;
use throbber_widgets_tui::ThrobberState;

use super::model::{config::ConfigModel, logs::LogModel, taskinstances::TaskInstanceModel};

#[derive(Debug)]
pub struct App {
    pub config: FlowrsConfig,
    pub environment_state: EnvironmentStateContainer,
    pub navigation: Navigation,
    pub dags: DagModel,
    pub configs: ConfigModel,
    pub dagruns: DagRunModel,
    pub task_instances: TaskInstanceModel,
    pub logs: LogModel,
    pub ticks: u32,
    pub loading: bool,
    pub throbber_state: ThrobberState,
    /// Global warning popup shown on startup (e.g., legacy config conflict)
    pub warning_popup: Option<WarningPopup>,
    /// Whether the terminal window has focus (used to pause refreshes when unfocused)
    pub focused: bool,
}

impl App {
    #[allow(dead_code, reason = "constructor used only in tests")]
    pub fn new(config: FlowrsConfig) -> Self {
        Self::new_with_errors_and_warnings(config, vec![], vec![])
    }

    pub fn new_with_errors_and_warnings(
        config: FlowrsConfig,
        errors: Vec<String>,
        warnings: Vec<String>,
    ) -> Self {
        let servers = config.servers.clone();
        let active_server = config
            .active_server
            .clone()
            .filter(|name| servers.iter().any(|server| &server.name == name));

        let warning_popup = if warnings.is_empty() {
            None
        } else {
            Some(WarningPopup::new(warnings))
        };

        let poll_tick_multiplier = config.poll_tick_multiplier();

        Self {
            config,
            environment_state: EnvironmentStateContainer::new(),
            navigation: Navigation::new(active_server),
            dags: DagModel::new(poll_tick_multiplier),
            configs: ConfigModel::new_with_errors(&servers, errors),
            dagruns: DagRunModel::new(poll_tick_multiplier),
            task_instances: TaskInstanceModel::new(poll_tick_multiplier),
            logs: LogModel::new(poll_tick_multiplier),
            ticks: 0,
            loading: true,
            throbber_state: ThrobberState::default(),
            warning_popup,
            focused: true,
        }
    }

    /// Route an event to the currently active panel's `update`.
    ///
    /// Taking `&mut self` lets the panel field and `navigation` be borrowed as
    /// disjoint fields, so the path does not need to be cloned per event.
    pub fn update_active_panel(
        &mut self,
        event: &FlowrsEvent,
    ) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        let path = self.navigation.path();
        match self.navigation.level() {
            Level::Environments => self.configs.update(event, path),
            Level::Dags => self.dags.update(event, path),
            Level::DagRuns => self.dagruns.update(event, path),
            Level::TaskInstances => self.task_instances.update(event, path),
            Level::Logs => self.logs.update(event, path),
        }
    }

    /// Show an error popup on whichever panel is currently active.
    pub fn show_error(&mut self, errors: Vec<String>) {
        match self.navigation.level() {
            Level::Environments => self.configs.popup.show_error(errors),
            Level::Dags => self.dags.popup.show_error(errors),
            Level::DagRuns => self.dagruns.popup.show_error(errors),
            Level::TaskInstances => self.task_instances.popup.show_error(errors),
            Level::Logs => self.logs.error_popup = Some(ErrorPopup::from_strings(errors)),
        }
    }

    /// Move forward onto the item under the active panel's cursor, and
    /// return the refreshes the new level needs.
    pub fn go_forward(&mut self) -> Vec<WorkerMessage> {
        let selection = match self.navigation.level() {
            Level::Environments => self.configs.selection(),
            Level::Dags => self.dags.selection(),
            Level::DagRuns => self.dagruns.selection(),
            Level::TaskInstances => self.task_instances.selection(),
            Level::Logs => None,
        };
        let Some(selection) = selection else {
            return vec![];
        };

        if let Selection::Environment(name) = &selection {
            if let Err(e) = self.activate_environment(name) {
                self.configs.popup.show_error(vec![e.to_string()]);
                return vec![];
            }
        }

        let step = self.navigation.select(&selection);
        self.apply(&step);
        step.messages
    }

    /// Move back one level.
    pub fn go_back(&mut self) {
        let step = self.navigation.back();
        self.apply(&step);
    }

    /// Make the environment the active one, creating its client if needed.
    fn activate_environment(&mut self, name: &str) -> anyhow::Result<()> {
        let config = self
            .configs
            .table
            .all()
            .iter()
            .find(|config| config.name == name)
            .ok_or_else(|| anyhow::anyhow!("Configuration '{name}' not found"))?;
        self.environment_state.activate(config)?;
        self.config.active_server = Some(name.to_string());
        Ok(())
    }

    /// Drop the content of every level the move changed, then show the
    /// current level from the cache.
    fn apply(&mut self, step: &Move) {
        if let Some(changed_at) = step.changed_at {
            if changed_at <= Level::Dags {
                self.dags.reset();
            }
            if changed_at <= Level::DagRuns {
                self.dagruns.reset();
            }
            if changed_at <= Level::TaskInstances {
                self.task_instances.reset();
            }
            if changed_at <= Level::Logs {
                self.logs.reset();
            }
        }
        self.sync_panel(self.navigation.level());
    }
}

#[cfg(test)]
mod tests {
    use flowrs_config::FlowrsConfig;

    use super::App;
    use crate::airflow::graph::TaskGraph;
    use crate::airflow::model::common::{Dag, DagRun, EnvironmentKey};
    use crate::app::model::Popup;
    use crate::app::navigation::Level;

    const CONFIG: &str = r#"
        active_server = "prod"

        [[servers]]
        name = "prod"
        endpoint = "http://localhost:8080"
        [servers.auth.Basic]
        username = "airflow"
        password = "airflow"

        [[servers]]
        name = "dev"
        endpoint = "http://localhost:8081"
        [servers.auth.Basic]
        username = "airflow"
        password = "airflow"

        [[servers]]
        name = "broken"
        endpoint = "http://airflow.example.com"
        [servers.auth.Cookie]
        cookie = "session=abc123"
    "#;

    fn dag(dag_id: &str) -> Dag {
        Dag {
            dag_id: dag_id.into(),
            ..Default::default()
        }
    }

    fn dag_run(dag_id: &str, dag_run_id: &str) -> DagRun {
        DagRun {
            dag_id: dag_id.into(),
            dag_run_id: dag_run_id.into(),
            ..Default::default()
        }
    }

    /// An app on the DAG list of `prod`, whose cache holds DAGs `a` and `b`,
    /// each with one run.
    fn app_on_dag_list() -> App {
        let mut app = App::new(FlowrsConfig::parse_toml(CONFIG).unwrap());
        let prod = app.configs.table.all()[0].clone();
        app.environment_state.activate(&prod).unwrap();
        let env = app
            .environment_state
            .environments
            .get_mut(&EnvironmentKey::from("prod"))
            .unwrap();
        env.replace_dags(vec![dag("a"), dag("b")]);
        env.replace_dag_runs(&"a".into(), vec![dag_run("a", "run_a")]);
        env.replace_dag_runs(&"b".into(), vec![dag_run("b", "run_b")]);
        app.sync_panel(Level::Dags);
        app
    }

    fn put_cursor_on(app: &mut App, row: usize) {
        let table_state = match app.navigation.level() {
            Level::Environments => app.configs.table.state_mut(),
            Level::Dags => app.dags.table.state_mut(),
            Level::DagRuns => app.dagruns.table.state_mut(),
            Level::TaskInstances => app.task_instances.table.state_mut(),
            Level::Logs => unreachable!("the log view has no rows"),
        };
        table_state.select(Some(row));
    }

    #[test]
    fn entering_another_dag_drops_the_previous_task_graph() {
        let mut app = app_on_dag_list();
        put_cursor_on(&mut app, 0);
        app.go_forward();
        put_cursor_on(&mut app, 0);
        app.go_forward();
        app.task_instances.task_graph = Some(TaskGraph::default());

        app.go_back();
        app.go_back();
        put_cursor_on(&mut app, 1);
        app.go_forward();

        assert_eq!(app.navigation.level(), Level::DagRuns);
        assert!(app.task_instances.task_graph.is_none());
        let runs: Vec<_> = app
            .dagruns
            .table
            .all()
            .iter()
            .map(|r| &*r.dag_run_id)
            .collect();
        assert_eq!(runs, ["run_b"]);
    }

    #[test]
    fn switching_environment_activates_it_and_clears_every_level() {
        let mut app = app_on_dag_list();
        put_cursor_on(&mut app, 0);
        app.go_forward();
        app.go_back();
        app.go_back();
        assert_eq!(app.navigation.level(), Level::Environments);

        put_cursor_on(&mut app, 1);
        let messages = app.go_forward();

        assert_eq!(app.navigation.level(), Level::Dags);
        assert_eq!(
            app.environment_state.active_environment.as_deref(),
            Some("dev")
        );
        assert_eq!(app.config.active_server.as_deref(), Some("dev"));
        assert!(app.dags.table.all().is_empty());
        assert!(app.dagruns.table.all().is_empty());
        assert_eq!(app.navigation.path().dag_id(), None);
        assert_eq!(
            messages
                .iter()
                .map(|m| format!("{m:?}"))
                .collect::<Vec<_>>(),
            ["UpdateDagsAndStats"]
        );
    }

    #[test]
    fn an_environment_whose_client_cannot_be_built_is_not_entered() {
        let mut app = app_on_dag_list();
        app.go_back();

        put_cursor_on(&mut app, 2);
        let messages = app.go_forward();

        assert!(messages.is_empty());
        assert_eq!(app.navigation.level(), Level::Environments);
        assert_eq!(
            app.environment_state.active_environment.as_deref(),
            Some("prod")
        );
        assert!(matches!(app.configs.popup, Popup::Error(_)));
    }
}
