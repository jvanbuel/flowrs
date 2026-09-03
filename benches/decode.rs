//! Decoding DAG-list responses from the v1 (Airflow 2) and v2 (Airflow 3)
//! APIs. The fixtures are serialized from the response models themselves, so
//! they track the schema automatically.

mod common;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use flowrs_airflow::client::v1::model::dag::{DagCollectionResponse, DagResponse, DagTagResponse};
use flowrs_airflow::client::v2::model::dag::{Dag as V2Dag, DagList as V2DagList, Tag as V2Tag};
use std::hint::black_box;
use time::macros::datetime;

const SIZES: [usize; 3] = [50, 500, 5_000];

fn v1_json(n: usize) -> String {
    let dags = (0..n)
        .map(|i| DagResponse {
            dag_id: format!("dag_{i:05}_pipeline"),
            dag_display_name: Some(format!("Pipeline {i}")),
            is_paused: Some(i % 7 == 0),
            is_active: Some(true),
            last_parsed_time: Some(datetime!(2026-01-01 00:00:00 UTC)),
            fileloc: format!("/opt/airflow/dags/pipeline_{i}.py"),
            file_token: format!("token-{i}"),
            owners: vec!["team-data".to_string()],
            description: Some("Synthetic DAG".to_string()),
            timetable_description: Some("At 00:00".to_string()),
            tags: Some(vec![DagTagResponse {
                name: "etl".to_string(),
            }]),
            max_active_tasks: Some(16),
            max_active_runs: Some(1),
            has_task_concurrency_limits: Some(false),
            has_import_errors: Some(false),
            next_dagrun_create_after: Some(datetime!(2026-01-01 01:00:00 UTC)),
            ..DagResponse::default()
        })
        .collect();
    serde_json::to_string(&DagCollectionResponse {
        dags,
        total_entries: i64::try_from(n).expect("fits"),
    })
    .expect("serialize v1 fixture")
}

fn v2_json(n: usize) -> String {
    let dags = (0..n)
        .map(|i| V2Dag {
            dag_id: format!("dag_{i:05}_pipeline"),
            dag_display_name: format!("Pipeline {i}"),
            is_paused: i % 7 == 0,
            last_parsed_time: Some(datetime!(2026-01-01 00:00:00 UTC)),
            fileloc: format!("/opt/airflow/dags/pipeline_{i}.py"),
            description: Some("Synthetic DAG".to_string()),
            timetable_description: Some("At 00:00".to_string()),
            tags: vec![V2Tag {
                name: "etl".to_string(),
            }],
            max_active_tasks: 16,
            max_active_runs: Some(1),
            next_dagrun_run_after: Some(datetime!(2026-01-01 01:00:00 UTC)),
            owners: vec!["team-data".to_string()],
            ..V2Dag::default()
        })
        .collect();
    serde_json::to_string(&V2DagList {
        dags,
        total_entries: i64::try_from(n).expect("fits"),
    })
    .expect("serialize v2 fixture")
}

fn decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode/dag_list");
    for n in SIZES {
        let v1 = v1_json(n);
        let v2 = v2_json(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::new("v1", n), &v1, |b, body| {
            b.iter(|| serde_json::from_str::<DagCollectionResponse>(black_box(body)));
        });
        group.bench_with_input(BenchmarkId::new("v2", n), &v2, |b, body| {
            b.iter(|| serde_json::from_str::<V2DagList>(black_box(body)));
        });
    }
    group.finish();
}

criterion_group!(timing, decode);
criterion_main!(timing);
