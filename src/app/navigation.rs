//! Where the user is: what they have selected (the [`Path`]) and which part of
//! it they are looking at (the [`Level`]).
//!
//! Panels report what is under their cursor as a [`Selection`]; [`Navigation`]
//! decides how the path changes, and returns a [`Move`] saying which levels
//! now show different content and which refreshes to request.

use log::debug;

use crate::airflow::model::common::{DagId, DagRunId, TaskId};
use crate::app::worker::WorkerMessage;

/// Which part of the path the user is looking at, from the outside in.
///
/// Ordered so that `a <= b` means `a` is at or above `b`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Environments,
    Dags,
    DagRuns,
    TaskInstances,
    Logs,
}

impl Level {
    const fn deeper(self) -> Self {
        match self {
            Level::Environments => Level::Dags,
            Level::Dags => Level::DagRuns,
            Level::DagRuns => Level::TaskInstances,
            Level::TaskInstances | Level::Logs => Level::Logs,
        }
    }

    const fn shallower(self) -> Self {
        match self {
            Level::Environments | Level::Dags => Level::Environments,
            Level::DagRuns => Level::Dags,
            Level::TaskInstances => Level::DagRuns,
            Level::Logs => Level::TaskInstances,
        }
    }
}

/// What the user has selected. Encoded as an enum to enforce the strict
/// hierarchy: environment > dag > `dag_run` > task instance. It is impossible
/// to have, e.g., a `task_id` without a `dag_id`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Path {
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
    TaskInstance {
        environment: String,
        dag_id: DagId,
        dag_run_id: DagRunId,
        task_id: TaskId,
        /// The task instance's latest try when it was selected; logs are
        /// fetched for every try up to this one.
        try_number: u32,
    },
}

impl Path {
    pub fn environment(&self) -> Option<&String> {
        match self {
            Path::None => Option::None,
            Path::Environment { environment, .. }
            | Path::Dag { environment, .. }
            | Path::DagRun { environment, .. }
            | Path::TaskInstance { environment, .. } => Some(environment),
        }
    }

    pub fn dag_id(&self) -> Option<&DagId> {
        match self {
            Path::None | Path::Environment { .. } => Option::None,
            Path::Dag { dag_id, .. }
            | Path::DagRun { dag_id, .. }
            | Path::TaskInstance { dag_id, .. } => Some(dag_id),
        }
    }

    pub fn dag_run_id(&self) -> Option<&DagRunId> {
        match self {
            Path::None | Path::Environment { .. } | Path::Dag { .. } => Option::None,
            Path::DagRun { dag_run_id, .. } | Path::TaskInstance { dag_run_id, .. } => {
                Some(dag_run_id)
            }
        }
    }

    pub fn task_id(&self) -> Option<&TaskId> {
        match self {
            Path::TaskInstance { task_id, .. } => Some(task_id),
            _ => Option::None,
        }
    }

    pub fn try_number(&self) -> Option<u32> {
        match self {
            Path::TaskInstance { try_number, .. } => Some(*try_number),
            _ => Option::None,
        }
    }
}

/// What a panel has under its cursor when the user moves forward.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Selection {
    Environment(String),
    Dag(DagId),
    DagRun(DagRunId),
    TaskInstance { task_id: TaskId, try_number: u32 },
}

/// The outcome of a navigation step.
#[derive(Debug, Default)]
pub struct Move {
    /// The shallowest level whose content changed. Every level at or below
    /// it must drop what it was showing; `None` means nothing changed.
    pub changed_at: Option<Level>,
    /// Refreshes to request for the level the user arrived at.
    pub messages: Vec<WorkerMessage>,
}

/// How a selection would change the path.
enum PathAfter {
    /// The selection does not belong to the current level.
    Unfit,
    /// The selection is what is already selected; the deeper path is kept.
    Same,
    /// The selection replaces the path from this level down.
    Replaced(Path),
}

#[derive(Debug)]
pub struct Navigation {
    path: Path,
    level: Level,
}

impl Navigation {
    /// Start in `environment` if one is given, otherwise at the environment list.
    pub fn new(environment: Option<String>) -> Self {
        match environment {
            Some(environment) => Self {
                path: Path::Environment { environment },
                level: Level::Dags,
            },
            None => Self {
                path: Path::None,
                level: Level::Environments,
            },
        }
    }

    pub const fn path(&self) -> &Path {
        &self.path
    }

    pub const fn level(&self) -> Level {
        self.level
    }

    /// Move one level deeper by selecting the item under the cursor.
    ///
    /// Selecting the item the user came back from keeps what was selected
    /// below it; selecting a different item drops it. A selection that does
    /// not belong to the current level is ignored.
    pub fn select(&mut self, selection: &Selection) -> Move {
        let new_path = match self.path_after(selection) {
            PathAfter::Unfit => {
                debug!("Ignoring {selection:?} at level {:?}", self.level);
                return Move::default();
            }
            PathAfter::Same => None,
            PathAfter::Replaced(path) => Some(path),
        };

        self.level = self.level.deeper();
        let changed_at = new_path.map(|path| {
            self.path = path;
            self.level
        });
        Move {
            changed_at,
            messages: self.refresh_messages(),
        }
    }

    /// The path after selecting `selection` at the current level.
    fn path_after(&self, selection: &Selection) -> PathAfter {
        let path = &self.path;
        let replaced = match (self.level, selection) {
            (Level::Environments, Selection::Environment(environment)) => {
                (path.environment() != Some(environment)).then(|| Path::Environment {
                    environment: environment.clone(),
                })
            }
            (Level::Dags, Selection::Dag(dag_id)) => {
                let Some(environment) = path.environment() else {
                    return PathAfter::Unfit;
                };
                (path.dag_id() != Some(dag_id)).then(|| Path::Dag {
                    environment: environment.clone(),
                    dag_id: dag_id.clone(),
                })
            }
            (Level::DagRuns, Selection::DagRun(dag_run_id)) => {
                let (Some(environment), Some(dag_id)) = (path.environment(), path.dag_id()) else {
                    return PathAfter::Unfit;
                };
                (path.dag_run_id() != Some(dag_run_id)).then(|| Path::DagRun {
                    environment: environment.clone(),
                    dag_id: dag_id.clone(),
                    dag_run_id: dag_run_id.clone(),
                })
            }
            (
                Level::TaskInstances,
                Selection::TaskInstance {
                    task_id,
                    try_number,
                },
            ) => {
                let (Some(environment), Some(dag_id), Some(dag_run_id)) =
                    (path.environment(), path.dag_id(), path.dag_run_id())
                else {
                    return PathAfter::Unfit;
                };
                (path.task_id() != Some(task_id) || path.try_number() != Some(*try_number)).then(
                    || Path::TaskInstance {
                        environment: environment.clone(),
                        dag_id: dag_id.clone(),
                        dag_run_id: dag_run_id.clone(),
                        task_id: task_id.clone(),
                        try_number: *try_number,
                    },
                )
            }
            _ => return PathAfter::Unfit,
        };
        replaced.map_or(PathAfter::Same, PathAfter::Replaced)
    }

    /// Move one level up. The path is kept, so moving forward onto the same
    /// item again finds the deeper selections where the user left them.
    pub fn back(&mut self) -> Move {
        self.level = self.level.shallower();
        Move::default()
    }

    /// The fetches that load the content of the current level.
    fn refresh_messages(&self) -> Vec<WorkerMessage> {
        let path = &self.path;
        match self.level {
            Level::Environments => vec![],
            Level::Dags => vec![WorkerMessage::UpdateDagsAndStats],
            Level::DagRuns => path
                .dag_id()
                .map(|dag_id| WorkerMessage::UpdateDagRuns {
                    dag_id: dag_id.clone(),
                })
                .into_iter()
                .collect(),
            Level::TaskInstances => match (path.dag_id(), path.dag_run_id()) {
                (Some(dag_id), Some(dag_run_id)) => vec![
                    WorkerMessage::UpdateTasks {
                        dag_id: dag_id.clone(),
                    },
                    WorkerMessage::UpdateTaskInstances {
                        dag_id: dag_id.clone(),
                        dag_run_id: dag_run_id.clone(),
                    },
                ],
                _ => vec![],
            },
            Level::Logs => match (
                path.dag_id(),
                path.dag_run_id(),
                path.task_id(),
                path.try_number(),
            ) {
                (Some(dag_id), Some(dag_run_id), Some(task_id), Some(task_try)) => {
                    vec![WorkerMessage::UpdateTaskLogs {
                        dag_id: dag_id.clone(),
                        dag_run_id: dag_run_id.clone(),
                        task_id: task_id.clone(),
                        task_try,
                    }]
                }
                _ => vec![],
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn messages(step: &Move) -> Vec<String> {
        step.messages.iter().map(|m| format!("{m:?}")).collect()
    }

    fn task(task_id: &str, try_number: u32) -> Selection {
        Selection::TaskInstance {
            task_id: task_id.into(),
            try_number,
        }
    }

    /// Navigation already inside DAG run `run_1` of `dag_a` in `prod`.
    fn in_dag_run() -> Navigation {
        let mut nav = Navigation::new(Some("prod".to_string()));
        nav.select(&Selection::Dag("dag_a".into()));
        nav.select(&Selection::DagRun("run_1".into()));
        nav
    }

    #[test]
    fn starts_at_the_dag_list_of_the_active_environment() {
        let nav = Navigation::new(Some("prod".to_string()));
        assert_eq!(nav.level(), Level::Dags);
        assert_eq!(nav.path().environment().map(String::as_str), Some("prod"));

        let nav = Navigation::new(None);
        assert_eq!(nav.level(), Level::Environments);
        assert_eq!(nav.path(), &Path::None);
    }

    #[test]
    fn each_selection_goes_one_level_deeper_and_requests_its_content() {
        let mut nav = Navigation::new(None);

        let step = nav.select(&Selection::Environment("prod".to_string()));
        assert_eq!(
            (nav.level(), step.changed_at),
            (Level::Dags, Some(Level::Dags))
        );
        assert_eq!(messages(&step), ["UpdateDagsAndStats"]);

        let step = nav.select(&Selection::Dag("dag_a".into()));
        assert_eq!(step.changed_at, Some(Level::DagRuns));
        assert_eq!(
            messages(&step),
            [r#"UpdateDagRuns { dag_id: DagId("dag_a") }"#]
        );

        let step = nav.select(&Selection::DagRun("run_1".into()));
        assert_eq!(step.changed_at, Some(Level::TaskInstances));
        assert_eq!(
            messages(&step),
            [
                r#"UpdateTasks { dag_id: DagId("dag_a") }"#,
                r#"UpdateTaskInstances { dag_id: DagId("dag_a"), dag_run_id: DagRunId("run_1") }"#,
            ]
        );

        let step = nav.select(&task("extract", 2));
        assert_eq!(
            (nav.level(), step.changed_at),
            (Level::Logs, Some(Level::Logs))
        );
        assert_eq!(
            messages(&step),
            [
                r#"UpdateTaskLogs { dag_id: DagId("dag_a"), dag_run_id: DagRunId("run_1"), task_id: TaskId("extract"), task_try: 2 }"#
            ]
        );
    }

    #[test]
    fn going_back_keeps_the_path_and_changes_nothing() {
        let mut nav = in_dag_run();
        let path = nav.path().clone();

        let step = nav.back();
        assert_eq!(nav.level(), Level::DagRuns);
        assert_eq!(step.changed_at, None);
        assert!(step.messages.is_empty());
        assert_eq!(nav.path(), &path);

        nav.back();
        nav.back();
        nav.back();
        assert_eq!(nav.level(), Level::Environments);
    }

    #[test]
    fn reselecting_the_same_item_keeps_the_deeper_path() {
        let mut nav = in_dag_run();
        nav.back();
        nav.back();

        let step = nav.select(&Selection::Dag("dag_a".into()));

        assert_eq!(step.changed_at, None);
        assert_eq!(nav.path().dag_run_id().map(|id| &**id), Some("run_1"));
        // The level's content is still refreshed.
        assert_eq!(
            messages(&step),
            [r#"UpdateDagRuns { dag_id: DagId("dag_a") }"#]
        );
    }

    #[test]
    fn selecting_a_different_item_drops_the_deeper_path() {
        let mut nav = in_dag_run();
        nav.back();
        nav.back();

        let step = nav.select(&Selection::Dag("dag_b".into()));

        assert_eq!(step.changed_at, Some(Level::DagRuns));
        assert_eq!(nav.path().dag_id().map(|id| &**id), Some("dag_b"));
        assert_eq!(nav.path().dag_run_id(), None);
    }

    #[test]
    fn switching_environment_changes_everything_from_the_dag_list_down() {
        let mut nav = in_dag_run();
        nav.back();
        nav.back();
        nav.back();

        let step = nav.select(&Selection::Environment("dev".to_string()));

        assert_eq!(step.changed_at, Some(Level::Dags));
        assert_eq!(
            nav.path(),
            &Path::Environment {
                environment: "dev".to_string()
            }
        );
    }

    #[test]
    fn a_new_try_of_the_same_task_counts_as_a_different_selection() {
        let mut nav = in_dag_run();
        nav.select(&task("extract", 1));
        nav.back();

        assert_eq!(nav.select(&task("extract", 1)).changed_at, None);
        nav.back();
        assert_eq!(
            nav.select(&task("extract", 2)).changed_at,
            Some(Level::Logs)
        );
    }

    #[test]
    fn a_selection_from_another_level_is_ignored() {
        let mut nav = in_dag_run();
        let path = nav.path().clone();

        let step = nav.select(&Selection::Dag("dag_b".into()));

        assert_eq!(nav.level(), Level::TaskInstances);
        assert_eq!(nav.path(), &path);
        assert_eq!(step.changed_at, None);
        assert!(step.messages.is_empty());
    }

    #[test]
    fn nothing_is_deeper_than_logs() {
        let mut nav = in_dag_run();
        nav.select(&task("extract", 1));

        let step = nav.select(&task("load", 1));

        assert_eq!(nav.level(), Level::Logs);
        assert_eq!(nav.path().task_id().map(|id| &**id), Some("extract"));
        assert!(step.messages.is_empty());
    }
}
