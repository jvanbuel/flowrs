use std::collections::HashMap;
use std::sync::Arc;

use flowrs_config::AirflowConfig;

use crate::airflow::client::FlowrsClient;
use crate::airflow::graph::{sort_task_instances, TaskGraph};
use crate::airflow::model::common::{
    Dag, DagId, DagRun, DagRunId, DagRunState, DagStatistic, EnvironmentKey, Log, TaskId,
    TaskInstance, TaskInstanceState,
};

/// Flat, request-keyed cache for a single Airflow environment.
///
/// Each collection is keyed by the parameters of the API request that produced
/// it. `task_instances` and `task_logs` use composite tuple keys instead of
/// nested `HashMap`s, giving single-lookup access and eliminating intermediate
/// allocations.
#[derive(Debug, Clone)]
pub struct EnvironmentData {
    pub client: Arc<FlowrsClient>,

    /// Result of `list_dags()` — sorted alphabetically by `dag_id` on write.
    fetched_dags: Arc<[Dag]>,
    /// `fetched_dags` with pending patches applied; what panels show.
    pub dags: Arc<[Dag]>,

    /// Result of `get_dag_stats(dag_ids)` — keyed per DAG.
    dag_stats: Arc<HashMap<DagId, Vec<DagStatistic>>>,

    /// Result of `list_dagruns(dag_id)` — keyed by `dag_id`, newest first.
    fetched_dag_runs: HashMap<DagId, Arc<[DagRun]>>,
    /// `fetched_dag_runs` with pending patches applied; what panels show.
    dag_runs: HashMap<DagId, Arc<[DagRun]>>,

    /// Result of `list_tasks(dag_id)`: the order task instances are shown in.
    task_graphs: HashMap<DagId, Arc<TaskGraph>>,

    /// Result of `list_task_instances(dag_id, dag_run_id)` — flat composite
    /// key, in task graph order once the DAG's graph is known.
    fetched_task_instances: HashMap<(DagId, DagRunId), Arc<[TaskInstance]>>,
    /// `fetched_task_instances` with pending patches applied; what panels show.
    task_instances: HashMap<(DagId, DagRunId), Arc<[TaskInstance]>>,

    /// Result of `get_task_logs(dag_id, dag_run_id, task_id, try)` — flat composite key.
    task_logs: HashMap<(DagId, DagRunId, TaskId), Arc<[Log]>>,

    /// Result of `get_dag_params(dag_id)` — cached param schemas for trigger popup.
    /// `Arc` so the per-sync copies into the panel models are pointer bumps,
    /// not deep clones of the (potentially large) schema JSON.
    pub dag_params: HashMap<DagId, Arc<serde_json::Value>>,

    /// Changes shown before the server confirmed them, re-applied on top of
    /// every fetch result until a fetch started after they were settled.
    pending: Vec<PendingPatch>,
}

impl EnvironmentData {
    pub fn new(client: Arc<FlowrsClient>) -> Self {
        Self {
            client,
            fetched_dags: Arc::default(),
            dags: Arc::default(),
            dag_stats: Arc::default(),
            fetched_dag_runs: HashMap::new(),
            dag_runs: HashMap::new(),
            task_graphs: HashMap::new(),
            fetched_task_instances: HashMap::new(),
            task_instances: HashMap::new(),
            task_logs: HashMap::new(),
            dag_params: HashMap::new(),
            pending: Vec::new(),
        }
    }

    /// Store a DAG's task graph and put its cached task instances in order.
    fn replace_task_graph(&mut self, dag_id: &DagId, graph: TaskGraph) {
        for ((instances_dag, _), instances) in &mut self.fetched_task_instances {
            if instances_dag == dag_id {
                let mut sorted = instances.to_vec();
                sort_task_instances(&mut sorted, &graph);
                *instances = sorted.into();
            }
        }
        self.task_graphs.insert(dag_id.clone(), Arc::new(graph));
    }

    /// Recompute what panels show: the fetched rows with every pending patch
    /// applied on top.
    fn rebuild_views(&mut self) {
        self.dags = Arc::clone(&self.fetched_dags);
        self.dag_runs.clone_from(&self.fetched_dag_runs);
        self.task_instances.clone_from(&self.fetched_task_instances);
        for pending in self.pending.clone() {
            pending.patch.apply_to(self);
        }
    }

    /// Update cached params for a DAG.
    pub fn update_dag_params(&mut self, dag_id: &DagId, params: serde_json::Value) {
        self.dag_params.insert(dag_id.clone(), Arc::new(params));
    }
}

/// Identifies a fetch: the environment it was started for, and when it was
/// started, so its result and pending patches can be ordered against each other.
#[derive(Clone, Debug)]
pub struct FetchTicket {
    environment: String,
    started: u64,
}

impl FetchTicket {
    /// The environment the fetch was started for.
    pub fn environment(&self) -> &str {
        &self.environment
    }
}

/// A fetch result, as handed to [`EnvironmentStateContainer::apply`].
#[derive(Debug)]
pub enum Fetched {
    Dags(Vec<Dag>),
    DagStats(Vec<(DagId, Vec<DagStatistic>)>),
    DagRuns {
        dag_id: DagId,
        dag_runs: Vec<DagRun>,
    },
    TaskInstances {
        dag_id: DagId,
        dag_run_id: DagRunId,
        task_instances: Vec<TaskInstance>,
    },
    TaskGraph {
        dag_id: DagId,
        graph: TaskGraph,
    },
    TaskLogs {
        dag_id: DagId,
        dag_run_id: DagRunId,
        task_id: TaskId,
        logs: Vec<Log>,
    },
}

#[derive(Clone, Debug)]
struct PendingPatch {
    patch: Patch,
    /// The last fetch started when the server confirmed the change; fetches
    /// started later already include it.
    settled_at: Option<u64>,
}

/// A change the user made that is shown before the server confirms it.
#[derive(Clone, Debug, PartialEq)]
pub enum Patch {
    DagPaused {
        dag_id: DagId,
        is_paused: bool,
    },
    DagRunState {
        dag_id: DagId,
        dag_run_id: DagRunId,
        state: DagRunState,
    },
    TaskInstanceState {
        dag_id: DagId,
        dag_run_id: DagRunId,
        task_id: TaskId,
        state: TaskInstanceState,
    },
}

impl Patch {
    fn apply_to(&self, env: &mut EnvironmentData) {
        match self {
            Patch::DagPaused { dag_id, is_paused } => {
                env.dags = patched(
                    &env.dags,
                    |dag| dag.dag_id == *dag_id,
                    |dag| {
                        dag.is_paused = *is_paused;
                    },
                );
            }
            Patch::DagRunState {
                dag_id,
                dag_run_id,
                state,
            } => {
                if let Some(runs) = env.dag_runs.get_mut(dag_id) {
                    *runs = patched(
                        runs,
                        |run| run.dag_run_id == *dag_run_id,
                        |run| {
                            run.state = state.clone();
                        },
                    );
                }
            }
            Patch::TaskInstanceState {
                dag_id,
                dag_run_id,
                task_id,
                state,
            } => {
                let key = (dag_id.clone(), dag_run_id.clone());
                if let Some(instances) = env.task_instances.get_mut(&key) {
                    *instances = patched(
                        instances,
                        |ti| ti.task_id == *task_id,
                        |ti| {
                            ti.state = Some(state.clone());
                        },
                    );
                }
            }
        }
    }
}

/// A copy of `rows` with `change` applied to the rows that match.
fn patched<T: Clone>(
    rows: &[T],
    matches: impl Fn(&T) -> bool,
    change: impl Fn(&mut T),
) -> Arc<[T]> {
    rows.iter()
        .cloned()
        .map(|mut row| {
            if matches(&row) {
                change(&mut row);
            }
            row
        })
        .collect()
}

/// Container for all environment states
#[derive(Debug, Clone)]
pub struct EnvironmentStateContainer {
    pub environments: HashMap<EnvironmentKey, EnvironmentData>,
    pub active_environment: Option<EnvironmentKey>,
    next_ticket: u64,
}

impl EnvironmentStateContainer {
    pub fn new() -> Self {
        Self {
            environments: HashMap::new(),
            active_environment: None,
            next_ticket: 0,
        }
    }

    /// Call before starting a fetch for `environment`; pass the ticket to
    /// `apply` with its result.
    pub fn begin_fetch(&mut self, environment: &str) -> FetchTicket {
        self.next_ticket += 1;
        FetchTicket {
            environment: environment.to_string(),
            started: self.next_ticket,
        }
    }

    /// Store a fetch result in the environment that requested it. Returns
    /// whether that environment is the active one, i.e. whether the visible
    /// panels should be refreshed.
    pub fn apply(&mut self, ticket: &FetchTicket, fetched: Fetched) -> bool {
        let environment = ticket.environment();
        let Some(env) = self.environments.get_mut(environment) else {
            return false;
        };
        // Store the fetched rows in display order; `rebuild_views` derives
        // what panels show from them.
        match fetched {
            Fetched::Dags(mut dags) => {
                dags.sort_by(|a, b| a.dag_id.cmp(&b.dag_id));
                env.fetched_dags = dags.into();
            }
            Fetched::DagStats(stats) => Arc::make_mut(&mut env.dag_stats).extend(stats),
            Fetched::DagRuns {
                dag_id,
                mut dag_runs,
            } => {
                // Newest first; a run without a logical date is placed by its
                // start date.
                dag_runs.sort_by(|a, b| {
                    b.logical_date
                        .or(b.start_date)
                        .cmp(&a.logical_date.or(a.start_date))
                });
                env.fetched_dag_runs.insert(dag_id, dag_runs.into());
            }
            Fetched::TaskInstances {
                dag_id,
                dag_run_id,
                mut task_instances,
            } => {
                if let Some(graph) = env.task_graphs.get(&dag_id) {
                    sort_task_instances(&mut task_instances, graph);
                }
                env.fetched_task_instances
                    .insert((dag_id, dag_run_id), task_instances.into());
            }
            Fetched::TaskGraph { dag_id, graph } => env.replace_task_graph(&dag_id, graph),
            Fetched::TaskLogs {
                dag_id,
                dag_run_id,
                task_id,
                logs,
            } => {
                env.task_logs
                    .insert((dag_id, dag_run_id, task_id), logs.into());
            }
        }
        env.pending.retain(|pending| {
            pending
                .settled_at
                .is_none_or(|settled| ticket.started <= settled)
        });
        env.rebuild_views();
        self.active_environment.as_deref() == Some(environment)
    }

    /// Show a change in the active environment before the server confirms it.
    pub fn patch(&mut self, patch: Patch) {
        let Some(env) = self
            .active_environment
            .as_ref()
            .and_then(|key| self.environments.get_mut(key))
        else {
            return;
        };
        patch.apply_to(env);
        env.pending.push(PendingPatch {
            patch,
            settled_at: None,
        });
    }

    /// Record the server's answer to a patched change. Once it succeeded,
    /// fetches started from now on carry the change themselves.
    /// If it failed, the change is undone.
    pub fn settle(&mut self, patch: &Patch, succeeded: bool) {
        let now = self.next_ticket;
        for env in self.environments.values_mut() {
            if succeeded {
                for pending in env.pending.iter_mut().filter(|p| p.patch == *patch) {
                    pending.settled_at = Some(now);
                }
            } else {
                env.pending.retain(|p| p.patch != *patch);
                env.rebuild_views();
            }
        }
    }

    /// The active environment's DAGs, by `dag_id`.
    pub fn dags(&self) -> Arc<[Dag]> {
        self.get_active_environment()
            .map(|env| Arc::clone(&env.dags))
            .unwrap_or_default()
    }

    /// The active environment's DAG statistics, per DAG.
    pub fn dag_stats(&self) -> Arc<HashMap<DagId, Vec<DagStatistic>>> {
        self.get_active_environment()
            .map(|env| Arc::clone(&env.dag_stats))
            .unwrap_or_default()
    }

    /// The active environment's runs of `dag_id`, newest first.
    pub fn dag_runs(&self, dag_id: &DagId) -> Arc<[DagRun]> {
        self.get_active_environment()
            .and_then(|env| env.dag_runs.get(dag_id))
            .cloned()
            .unwrap_or_default()
    }

    /// The active environment's task instances of one DAG run, in task graph
    /// order once the DAG's graph is known.
    pub fn task_instances(&self, dag_id: &DagId, dag_run_id: &DagRunId) -> Arc<[TaskInstance]> {
        self.get_active_environment()
            .and_then(|env| {
                env.task_instances
                    .get(&(dag_id.clone(), dag_run_id.clone()))
            })
            .cloned()
            .unwrap_or_default()
    }

    /// The active environment's task graph of `dag_id`, once fetched.
    pub fn task_graph(&self, dag_id: &DagId) -> Option<Arc<TaskGraph>> {
        self.get_active_environment()
            .and_then(|env| env.task_graphs.get(dag_id))
            .cloned()
    }

    /// The active environment's logs of a task instance, one per try.
    pub fn task_logs(&self, dag_id: &DagId, dag_run_id: &DagRunId, task_id: &TaskId) -> Arc<[Log]> {
        self.get_active_environment()
            .and_then(|env| {
                env.task_logs
                    .get(&(dag_id.clone(), dag_run_id.clone(), task_id.clone()))
            })
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_active_environment(&self) -> Option<&EnvironmentData> {
        self.active_environment
            .as_ref()
            .and_then(|key| self.environments.get(key))
    }

    pub fn set_active_environment(&mut self, key: EnvironmentKey) {
        if self.environments.contains_key(&key) {
            self.active_environment = Some(key);
        }
    }

    /// Make `config`'s environment the active one, creating its client the
    /// first time it is selected.
    pub fn activate(&mut self, config: &AirflowConfig) -> anyhow::Result<()> {
        let key = EnvironmentKey::from(config.name.clone());
        if !self.environments.contains_key(&key) {
            let client = FlowrsClient::new(config)
                .map_err(|e| anyhow::anyhow!("Failed to connect to '{key}': {e}"))?;
            self.environments
                .insert(key.clone(), EnvironmentData::new(Arc::new(client)));
        }
        self.active_environment = Some(key);
        Ok(())
    }
}

impl Default for EnvironmentStateContainer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use flowrs_config::FlowrsConfig;
    use time::macros::datetime;
    use time::OffsetDateTime;

    use super::*;
    use crate::airflow::graph::TaskGraph;
    use crate::airflow::model::common::Task;

    /// A cache with `prod` (active) and `dev`.
    fn cache() -> EnvironmentStateContainer {
        let config = FlowrsConfig::parse_toml(
            r#"
            [[servers]]
            name = "dev"
            endpoint = "http://localhost:8081"
            [servers.auth.Basic]
            username = "airflow"
            password = "airflow"

            [[servers]]
            name = "prod"
            endpoint = "http://localhost:8080"
            [servers.auth.Basic]
            username = "airflow"
            password = "airflow"
            "#,
        )
        .unwrap();
        let mut cache = EnvironmentStateContainer::new();
        for server in &config.servers {
            cache.activate(server).unwrap();
        }
        cache
    }

    fn run(dag_run_id: &str, logical_date: OffsetDateTime) -> DagRun {
        DagRun {
            dag_id: "a".into(),
            dag_run_id: dag_run_id.into(),
            logical_date: Some(logical_date),
            ..Default::default()
        }
    }

    fn run_ids(cache: &EnvironmentStateContainer) -> Vec<String> {
        cache
            .dag_runs(&"a".into())
            .iter()
            .map(|r| r.dag_run_id.to_string())
            .collect()
    }

    #[test]
    fn fetched_dag_runs_are_listed_newest_first() {
        let mut cache = cache();
        let ticket = cache.begin_fetch("prod");

        cache.apply(
            &ticket,
            Fetched::DagRuns {
                dag_id: "a".into(),
                dag_runs: vec![
                    run("monday", datetime!(2026-10-05 00:00 UTC)),
                    run("tuesday", datetime!(2026-10-06 00:00 UTC)),
                ],
            },
        );

        assert_eq!(run_ids(&cache), ["tuesday", "monday"]);
    }

    #[test]
    fn a_result_for_an_inactive_environment_does_not_reach_the_active_one() {
        let mut cache = cache();
        let ticket = cache.begin_fetch("dev");

        let visible = cache.apply(
            &ticket,
            Fetched::DagRuns {
                dag_id: "a".into(),
                dag_runs: vec![run("from_dev", datetime!(2026-10-05 00:00 UTC))],
            },
        );

        assert!(!visible);
        assert_eq!(run_ids(&cache), Vec::<String>::new());
        cache.active_environment = Some("dev".into());
        assert_eq!(run_ids(&cache), ["from_dev"]);
    }

    fn fetch_runs(cache: &mut EnvironmentStateContainer, ticket: &FetchTicket, runs: Vec<DagRun>) {
        cache.apply(
            ticket,
            Fetched::DagRuns {
                dag_id: "a".into(),
                dag_runs: runs,
            },
        );
    }

    fn mark_monday_failed() -> Patch {
        Patch::DagRunState {
            dag_id: "a".into(),
            dag_run_id: "monday".into(),
            state: DagRunState::Failed,
        }
    }

    fn monday_state(cache: &EnvironmentStateContainer) -> DagRunState {
        cache.dag_runs(&"a".into())[0].state.clone()
    }

    #[test]
    fn a_marked_dag_run_shows_its_new_state_at_once() {
        let mut cache = cache();
        let ticket = cache.begin_fetch("prod");
        fetch_runs(
            &mut cache,
            &ticket,
            vec![run("monday", datetime!(2026-10-05 00:00 UTC))],
        );

        cache.patch(mark_monday_failed());

        assert_eq!(monday_state(&cache), DagRunState::Failed);
    }

    #[test]
    fn a_refresh_already_in_flight_keeps_the_mark() {
        let mut cache = cache();
        let first = cache.begin_fetch("prod");
        fetch_runs(
            &mut cache,
            &first,
            vec![run("monday", datetime!(2026-10-05 00:00 UTC))],
        );
        let in_flight = cache.begin_fetch("prod");

        cache.patch(mark_monday_failed());
        fetch_runs(
            &mut cache,
            &in_flight,
            vec![run("monday", datetime!(2026-10-05 00:00 UTC))],
        );

        assert_eq!(monday_state(&cache), DagRunState::Failed);
    }

    #[test]
    fn a_refresh_started_after_the_mark_succeeded_shows_the_server_state() {
        let mut cache = cache();
        let first = cache.begin_fetch("prod");
        fetch_runs(
            &mut cache,
            &first,
            vec![run("monday", datetime!(2026-10-05 00:00 UTC))],
        );
        cache.patch(mark_monday_failed());
        let before_settling = cache.begin_fetch("prod");

        cache.settle(&mark_monday_failed(), true);
        let after_settling = cache.begin_fetch("prod");
        let mut succeeded = run("monday", datetime!(2026-10-05 00:00 UTC));
        succeeded.state = DagRunState::Success;
        fetch_runs(
            &mut cache,
            &before_settling,
            vec![run("monday", datetime!(2026-10-05 00:00 UTC))],
        );
        assert_eq!(monday_state(&cache), DagRunState::Failed);
        fetch_runs(&mut cache, &after_settling, vec![succeeded]);

        assert_eq!(monday_state(&cache), DagRunState::Success);
    }

    #[test]
    fn a_mark_the_server_rejected_is_undone() {
        let mut cache = cache();
        let first = cache.begin_fetch("prod");
        let mut running = run("monday", datetime!(2026-10-05 00:00 UTC));
        running.state = DagRunState::Running;
        fetch_runs(&mut cache, &first, vec![running]);
        cache.patch(mark_monday_failed());

        cache.settle(&mark_monday_failed(), false);

        assert_eq!(monday_state(&cache), DagRunState::Running);
    }

    #[test]
    fn a_paused_dag_shows_as_paused_at_once_and_after_a_stale_refresh() {
        let mut cache = cache();
        let dag = Dag {
            dag_id: "a".into(),
            ..Default::default()
        };
        let first = cache.begin_fetch("prod");
        cache.apply(&first, Fetched::Dags(vec![dag.clone()]));
        let in_flight = cache.begin_fetch("prod");

        cache.patch(Patch::DagPaused {
            dag_id: "a".into(),
            is_paused: true,
        });
        assert!(cache.dags()[0].is_paused);
        cache.apply(&in_flight, Fetched::Dags(vec![dag]));

        assert!(cache.dags()[0].is_paused);
    }

    fn task_instance(task_id: &str) -> TaskInstance {
        TaskInstance {
            dag_id: "a".into(),
            dag_run_id: "monday".into(),
            task_id: task_id.into(),
            ..Default::default()
        }
    }

    #[test]
    fn a_marked_task_instance_shows_its_new_state_at_once() {
        let mut cache = cache();
        let ticket = cache.begin_fetch("prod");
        cache.apply(
            &ticket,
            Fetched::TaskInstances {
                dag_id: "a".into(),
                dag_run_id: "monday".into(),
                task_instances: vec![task_instance("extract")],
            },
        );

        cache.patch(Patch::TaskInstanceState {
            dag_id: "a".into(),
            dag_run_id: "monday".into(),
            task_id: "extract".into(),
            state: TaskInstanceState::Failed,
        });

        let instances = cache.task_instances(&"a".into(), &"monday".into());
        assert_eq!(instances[0].state, Some(TaskInstanceState::Failed));
    }

    #[test]
    fn task_instances_follow_the_task_graph_even_when_it_arrives_later() {
        let mut cache = cache();
        let ticket = cache.begin_fetch("prod");
        cache.apply(
            &ticket,
            Fetched::TaskInstances {
                dag_id: "a".into(),
                dag_run_id: "monday".into(),
                task_instances: vec![task_instance("load"), task_instance("extract")],
            },
        );

        let tasks = [
            Task {
                task_id: "extract".to_string(),
                downstream_task_ids: vec!["load".to_string()],
            },
            Task {
                task_id: "load".to_string(),
                downstream_task_ids: vec![],
            },
        ];
        let graph_ticket = cache.begin_fetch("prod");
        cache.apply(
            &graph_ticket,
            Fetched::TaskGraph {
                dag_id: "a".into(),
                graph: TaskGraph::from_tasks(&tasks),
            },
        );

        let order: Vec<String> = cache
            .task_instances(&"a".into(), &"monday".into())
            .iter()
            .map(|ti| ti.task_id.to_string())
            .collect();
        assert_eq!(order, ["extract", "load"]);
    }

    #[test]
    fn a_dag_run_without_a_logical_date_is_placed_by_its_start_date() {
        let mut cache = cache();
        let ticket = cache.begin_fetch("prod");
        let manual = DagRun {
            dag_id: "a".into(),
            dag_run_id: "manual".into(),
            start_date: Some(datetime!(2026-10-06 00:00 UTC)),
            ..Default::default()
        };

        fetch_runs(
            &mut cache,
            &ticket,
            vec![run("monday", datetime!(2026-10-05 00:00 UTC)), manual],
        );

        assert_eq!(run_ids(&cache), ["manual", "monday"]);
    }
}
