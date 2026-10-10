//! Synthetic fixtures shared by the benches. No network access.

#![allow(dead_code, reason = "each bench uses a subset")]

use std::fmt::Write as _;
use std::sync::Arc;

use flowrs_airflow::config::default_timeout;
use flowrs_config::{AirflowAuth, AirflowConfig, AirflowVersion, FlowrsConfig, TokenSource};
use flowrs_tui::airflow::client::FlowrsClient;
use flowrs_tui::airflow::model::common::{
    Dag, DagId, DagRun, DagRunId, DagRunState, DagStatistic, EnvironmentKey, Log, Tag, TaskId,
    TaskInstance, TaskInstanceState,
};
use flowrs_tui::app::state::environment_state::EnvironmentData;
use flowrs_tui::app::state::{App, NavigationContext, Panel};
use time::macros::datetime;
use time::{Duration, OffsetDateTime};

pub const ENV: &str = "bench";
const BASE: OffsetDateTime = datetime!(2026-01-01 00:00:00 UTC);
const OWNERS: [&str; 4] = ["team-data", "team-ml", "team-infra", "airflow"];
const TAGS: [&str; 5] = ["etl", "hourly", "daily", "ml", "backfill"];
const OPERATORS: [&str; 3] = ["PythonOperator", "BashOperator", "KubernetesPodOperator"];

fn idx(i: usize) -> i64 {
    i64::try_from(i).expect("fixture index fits in i64")
}

pub fn dag_id(i: usize) -> DagId {
    DagId::from(format!("dag_{i:05}_pipeline"))
}

pub fn dag_run_id(i: usize) -> DagRunId {
    DagRunId::from(format!("scheduled__2026-01-01T00:00:00+00:00__{i:05}"))
}

pub fn task_id(i: usize) -> TaskId {
    TaskId::from(format!("task_{i:04}_transform"))
}

pub fn dags(n: usize) -> Vec<Dag> {
    (0..n)
        .map(|i| Dag {
            dag_id: dag_id(i),
            fileloc: format!("/opt/airflow/dags/pipeline_{i}.py"),
            is_paused: i.is_multiple_of(7),
            next_dagrun_create_after: Some(BASE + Duration::hours(1 + idx(i) % 24)),
            owners: vec![
                OWNERS[i % OWNERS.len()].to_string(),
                OWNERS[(i / 3) % OWNERS.len()].to_string(),
            ],
            tags: vec![Tag {
                name: TAGS[i % TAGS.len()].to_string(),
            }],
            timetable_description: Some("At 00:00".to_string()),
            ..Dag::default()
        })
        .collect()
}

pub fn dag_stats(n: usize) -> Vec<(DagId, Vec<DagStatistic>)> {
    (0..n)
        .map(|i| {
            let stat = |state, count: usize| DagStatistic {
                state,
                count: count as u64,
            };
            (
                dag_id(i),
                vec![
                    stat(DagRunState::Success, i % 50),
                    stat(DagRunState::Failed, i % 3),
                ],
            )
        })
        .collect()
}

pub fn dag_runs(dag_id: &DagId, n: usize) -> Vec<DagRun> {
    let states = [
        DagRunState::Success,
        DagRunState::Failed,
        DagRunState::Running,
    ];
    (0..n)
        .map(|i| {
            let logical = BASE - Duration::hours(idx(i));
            DagRun {
                dag_id: dag_id.clone(),
                dag_run_id: dag_run_id(i),
                logical_date: Some(logical),
                start_date: Some(logical + Duration::minutes(1)),
                end_date: Some(logical + Duration::minutes(5 + idx(i) % 40)),
                state: states[i % states.len()].clone(),
                ..DagRun::default()
            }
        })
        .collect()
}

pub fn task_instances(dag_id: &DagId, dag_run_id: &DagRunId, n: usize) -> Vec<TaskInstance> {
    let states = [
        TaskInstanceState::Success,
        TaskInstanceState::Failed,
        TaskInstanceState::Running,
    ];
    (0..n)
        .map(|i| {
            let start = BASE + Duration::seconds(idx(i) * 3);
            TaskInstance {
                task_id: task_id(i),
                dag_id: dag_id.clone(),
                dag_run_id: dag_run_id.clone(),
                start_date: Some(start),
                end_date: Some(start + Duration::seconds(10 + idx(i) % 120)),
                state: Some(states[i % states.len()].clone()),
                operator: Some(OPERATORS[i % OPERATORS.len()].to_string()),
                ..TaskInstance::default()
            }
        })
        .collect()
}

/// At least `bytes` of task-log lines; every 40th line is an ERROR.
pub fn log_content(bytes: usize) -> String {
    let mut out = String::with_capacity(bytes + 256);
    let mut i = 0usize;
    while out.len() < bytes {
        let level = if i.is_multiple_of(40) {
            "ERROR"
        } else {
            "INFO"
        };
        writeln!(
            out,
            "[2026-01-01T00:{:02}:{:02}.000+0000] {{taskinstance.py:1234}} {level} - step {i}: processed batch with 128 records",
            (i / 60) % 60,
            i % 60
        )
        .expect("write to String");
        i += 1;
    }
    out
}

pub fn logs(bytes: usize, tries: usize) -> Vec<Log> {
    (0..tries)
        .map(|_| Log {
            continuation_token: None,
            content: log_content(bytes),
        })
        .collect()
}

/// Sizes of the environment behind a benchmark `App`.
#[derive(Debug, Clone, Copy)]
pub struct Shape {
    pub dags: usize,
    pub dag_runs: usize,
    pub task_instances: usize,
    pub log_bytes: usize,
}

impl Shape {
    pub const fn dags(dags: usize) -> Self {
        Self {
            dags,
            dag_runs: 50,
            task_instances: 50,
            log_bytes: 64 * 1024,
        }
    }
}

/// An `App` with one active environment populated to `shape`.
pub fn app(shape: Shape) -> App {
    let server = AirflowConfig {
        name: ENV.to_string(),
        endpoint: "http://localhost:8080".to_string(),
        auth: AirflowAuth::Token(TokenSource::Static {
            token: "bench".to_string(),
        }),
        managed: None,
        version: AirflowVersion::V3,
        timeout_secs: default_timeout(),
        insecure: false,
    };
    let mut config = FlowrsConfig::new(&flowrs_tui::CONFIG_PATHS);
    config.servers = vec![server.clone()];
    config.active_server = Some(ENV.to_string());

    let mut app = App::new(config);
    app.ticks = 11; // past the splash screen
    let client = Arc::new(FlowrsClient::new(&server).expect("client"));
    let mut env = EnvironmentData::new(client);

    let (dag, run, task) = (dag_id(0), dag_run_id(0), task_id(0));
    env.replace_dags(dags(shape.dags));
    for (id, stats) in dag_stats(shape.dags) {
        env.update_dag_stats(&id, stats);
    }
    env.replace_dag_runs(&dag, dag_runs(&dag, shape.dag_runs));
    env.replace_task_instances(&dag, &run, task_instances(&dag, &run, shape.task_instances));
    env.replace_task_logs(&dag, &run, &task, logs(shape.log_bytes, 2));

    let key = EnvironmentKey::from(ENV);
    app.environment_state.environments.insert(key.clone(), env);
    app.environment_state.set_active_environment(key);
    app
}

/// Point navigation at the populated DAG / run / task and sync `panel`.
pub fn navigate_to(app: &mut App, panel: &Panel) {
    let environment = ENV.to_string();
    app.nav_context = match panel {
        Panel::Config | Panel::Dag => NavigationContext::Environment { environment },
        Panel::DAGRun => NavigationContext::Dag {
            environment,
            dag_id: dag_id(0),
        },
        Panel::TaskInstance => NavigationContext::DagRun {
            environment,
            dag_id: dag_id(0),
            dag_run_id: dag_run_id(0),
        },
        Panel::Logs => NavigationContext::Task {
            environment,
            dag_id: dag_id(0),
            dag_run_id: dag_run_id(0),
            task_id: task_id(0),
            task_try: 1,
        },
    };
    app.active_panel = panel.clone();
    app.loading = false;
    app.sync_panel(panel);
}
