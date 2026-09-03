//! Shared fixtures and an allocation-counting measurement for the benches.
//!
//! Each bench binary includes this module with `mod common;`, so the global
//! allocator below is installed exactly once per binary.

#![allow(
    dead_code,
    reason = "each bench binary uses a different subset of these helpers"
)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::fmt::Write as _;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use criterion::measurement::{Measurement, ValueFormatter};
use criterion::Throughput;
use flowrs_airflow::config::default_timeout;
use flowrs_config::{AirflowAuth, AirflowConfig, AirflowVersion, FlowrsConfig, TokenSource};
use flowrs_tui::airflow::client::FlowrsClient;
use flowrs_tui::airflow::model::common::{
    Dag, DagId, DagRun, DagRunId, DagRunState, DagStatistic, EnvironmentKey, Log, RunType, Tag,
    TaskId, TaskInstance, TaskInstanceState,
};
use flowrs_tui::app::state::environment_state::EnvironmentData;
use flowrs_tui::app::state::{App, NavigationContext, Panel};
use time::macros::datetime;
use time::{Duration, OffsetDateTime};

// ── Allocation counting ─────────────────────────────────────────

static ALLOCS: AtomicUsize = AtomicUsize::new(0);

/// Counts every allocation and reallocation, then delegates to the system
/// allocator. The counter is a relaxed atomic increment, which is cheap enough
/// not to distort the timing benches in the same binary.
struct CountingAlloc;

// SAFETY: every method delegates to `System`, which upholds the `GlobalAlloc`
// contract; the counter has no effect on the returned memory.
unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: forwarded unchanged to the system allocator.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: `ptr` was returned by `System.alloc` with this layout.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: `ptr` was returned by `System.alloc` with this layout.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

/// Criterion measurement that reports heap allocations per iteration instead
/// of wall time. Use it in a group configured with
/// `Criterion::default().with_measurement(AllocCount)`.
#[derive(Debug, Clone, Copy)]
pub struct AllocCount;

impl Measurement for AllocCount {
    type Intermediate = usize;
    type Value = usize;

    fn start(&self) -> usize {
        ALLOCS.load(Ordering::Relaxed)
    }

    fn end(&self, start: usize) -> usize {
        ALLOCS.load(Ordering::Relaxed).saturating_sub(start)
    }

    fn add(&self, a: &usize, b: &usize) -> usize {
        a + b
    }

    fn zero(&self) -> usize {
        0
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "allocation counts are far below 2^52"
    )]
    fn to_f64(&self, value: &usize) -> f64 {
        *value as f64
    }

    fn formatter(&self) -> &dyn ValueFormatter {
        &AllocFormatter
    }
}

struct AllocFormatter;

impl ValueFormatter for AllocFormatter {
    fn scale_values(&self, _typical: f64, _values: &mut [f64]) -> &'static str {
        "allocs"
    }

    fn scale_throughputs(
        &self,
        _typical: f64,
        _throughput: &Throughput,
        _values: &mut [f64],
    ) -> &'static str {
        "allocs"
    }

    fn scale_for_machines(&self, _values: &mut [f64]) -> &'static str {
        "allocs"
    }
}

// ── Fixtures ────────────────────────────────────────────────────

pub const ENV: &str = "bench";
const BASE: OffsetDateTime = datetime!(2026-01-01 00:00:00 UTC);
const OWNERS: [&str; 4] = ["team-data", "team-ml", "team-infra", "airflow"];
const TAGS: [&str; 5] = ["etl", "hourly", "daily", "ml", "backfill"];
const OPERATORS: [&str; 3] = ["PythonOperator", "BashOperator", "KubernetesPodOperator"];

/// Fixture index as an `i64` for date arithmetic.
fn idx(i: usize) -> i64 {
    i64::try_from(i).expect("fixture index fits in i64")
}

pub fn dag_id(i: usize) -> DagId {
    DagId::from(format!("dag_{i:05}_pipeline"))
}

pub fn dags(n: usize) -> Vec<Dag> {
    (0..n)
        .map(|i| Dag {
            dag_id: dag_id(i),
            dag_display_name: Some(format!("Pipeline {i}")),
            description: Some(format!("Synthetic DAG number {i} for benchmarking")),
            fileloc: format!("/opt/airflow/dags/pipeline_{i}.py"),
            is_paused: i.is_multiple_of(7),
            is_active: Some(true),
            has_import_errors: false,
            has_task_concurrency_limits: false,
            last_parsed_time: Some(BASE - Duration::minutes(idx(i) % 60)),
            last_expired: None,
            max_active_tasks: 16,
            max_active_runs: Some(1),
            next_dagrun_logical_date: Some(BASE + Duration::hours(idx(i) % 24)),
            next_dagrun_data_interval_start: Some(BASE),
            next_dagrun_data_interval_end: Some(BASE + Duration::hours(1)),
            next_dagrun_create_after: Some(BASE + Duration::hours(1 + idx(i) % 24)),
            owners: vec![
                OWNERS[i % OWNERS.len()].to_string(),
                OWNERS[(i / 3) % OWNERS.len()].to_string(),
            ],
            tags: vec![
                Tag {
                    name: TAGS[i % TAGS.len()].to_string(),
                },
                Tag {
                    name: TAGS[(i / 2) % TAGS.len()].to_string(),
                },
            ],
            file_token: format!("token-{i}"),
            timetable_description: Some("At 00:00".to_string()),
        })
        .collect()
}

pub fn dag_stats(n: usize) -> Vec<(DagId, Vec<DagStatistic>)> {
    (0..n)
        .map(|i| {
            (
                dag_id(i),
                vec![
                    DagStatistic {
                        state: DagRunState::Success,
                        count: (i % 50) as u64,
                    },
                    DagStatistic {
                        state: DagRunState::Failed,
                        count: (i % 3) as u64,
                    },
                    DagStatistic {
                        state: DagRunState::Running,
                        count: u64::from(i.is_multiple_of(2)),
                    },
                ],
            )
        })
        .collect()
}

pub fn dag_run_id(i: usize) -> DagRunId {
    DagRunId::from(format!("scheduled__2026-01-01T00:00:00+00:00__{i:05}"))
}

pub fn dag_runs(dag_id: &DagId, n: usize) -> Vec<DagRun> {
    let states = [
        DagRunState::Success,
        DagRunState::Failed,
        DagRunState::Running,
        DagRunState::Queued,
    ];
    (0..n)
        .map(|i| {
            let logical = BASE - Duration::hours(idx(i));
            DagRun {
                dag_id: dag_id.clone(),
                dag_run_id: dag_run_id(i),
                logical_date: Some(logical),
                data_interval_start: Some(logical),
                data_interval_end: Some(logical + Duration::hours(1)),
                start_date: Some(logical + Duration::minutes(1)),
                end_date: Some(logical + Duration::minutes(5 + idx(i) % 40)),
                last_scheduling_decision: None,
                run_type: if i.is_multiple_of(5) {
                    RunType::Manual
                } else {
                    RunType::Scheduled
                },
                state: states[i % states.len()].clone(),
                note: None,
                external_trigger: Some(i.is_multiple_of(5)),
            }
        })
        .collect()
}

pub fn task_id(i: usize) -> TaskId {
    TaskId::from(format!("task_{i:04}_transform"))
}

pub fn task_instances(dag_id: &DagId, dag_run_id: &DagRunId, n: usize) -> Vec<TaskInstance> {
    let states = [
        TaskInstanceState::Success,
        TaskInstanceState::Failed,
        TaskInstanceState::Running,
        TaskInstanceState::Queued,
    ];
    (0..n)
        .map(|i| {
            let start = BASE + Duration::seconds(idx(i) * 3);
            TaskInstance {
                task_id: task_id(i),
                dag_id: dag_id.clone(),
                dag_run_id: dag_run_id.clone(),
                logical_date: Some(BASE),
                start_date: Some(start),
                end_date: Some(start + Duration::seconds(10 + idx(i) % 120)),
                duration: Some(10.0 + f64::from(u8::try_from(i % 120).expect("< 256"))),
                state: Some(states[i % states.len()].clone()),
                try_number: 1,
                max_tries: 2,
                map_index: -1,
                hostname: Some("worker-0".to_string()),
                unixname: Some("airflow".to_string()),
                pool: "default_pool".to_string(),
                pool_slots: 1,
                queue: Some("default".to_string()),
                priority_weight: Some(1),
                operator: Some(OPERATORS[i % OPERATORS.len()].to_string()),
                queued_when: Some(start - Duration::seconds(1)),
                scheduled_when: None,
                pid: Some(1000 + idx(i)),
                note: None,
            }
        })
        .collect()
}

/// A log body of at least `bytes` bytes in Airflow's task-log line format.
/// Every 40th line is an ERROR so searches have something to find.
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
            "[2026-01-01T00:{:02}:{:02}.000+0000] {{taskinstance.py:1234}} {level} - step {i}: processed batch with 128 records from source table",
            (i / 60) % 60,
            i % 60
        )
        .expect("writing to a String cannot fail");
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

// ── App construction ────────────────────────────────────────────

fn server() -> AirflowConfig {
    AirflowConfig {
        name: ENV.to_string(),
        endpoint: "http://localhost:8080".to_string(),
        auth: AirflowAuth::Token(TokenSource::Static {
            token: "bench".to_string(),
        }),
        managed: None,
        version: AirflowVersion::V3,
        timeout_secs: default_timeout(),
        insecure: false,
    }
}

/// Sizes of the synthetic environment behind a benchmark `App`.
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

/// An `App` with one active environment whose cache is populated to `shape`.
/// No network access happens: the client is constructed but never used.
pub fn app(shape: Shape) -> App {
    let server = server();
    let mut config = FlowrsConfig::new(&flowrs_tui::CONFIG_PATHS);
    config.servers = vec![server.clone()];
    config.active_server = Some(ENV.to_string());

    let mut app = App::new(config);
    // draw_ui shows the splash screen for the first ten ticks.
    app.ticks = 11;
    let client = Arc::new(FlowrsClient::new(&server).expect("client for a static endpoint"));
    let mut env = EnvironmentData::new(client);

    env.replace_dags(dags(shape.dags));
    for (id, stats) in dag_stats(shape.dags) {
        env.update_dag_stats(&id, stats);
    }
    let first_dag = dag_id(0);
    let first_run = dag_run_id(0);
    let first_task = task_id(0);
    env.replace_dag_runs(&first_dag, dag_runs(&first_dag, shape.dag_runs));
    env.replace_task_instances(
        &first_dag,
        &first_run,
        task_instances(&first_dag, &first_run, shape.task_instances),
    );
    env.replace_task_logs(
        &first_dag,
        &first_run,
        &first_task,
        logs(shape.log_bytes, 2),
    );

    let key = EnvironmentKey::from(ENV);
    app.environment_state.environments.insert(key.clone(), env);
    app.environment_state.set_active_environment(key);
    app
}

/// Point the navigation context at the populated DAG / run / task and sync
/// every panel, so any panel can be rendered or re-synced from here.
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
