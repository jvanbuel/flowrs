//! `App::sync_panel`: the copy from the environment cache into a panel table,
//! which runs after every worker update and on every panel switch.

mod common;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use flowrs_tui::app::state::Panel;
use std::hint::black_box;

use common::{AllocCount, Shape};

const SIZES: [usize; 3] = [100, 1_000, 5_000];

fn sync_dags(c: &mut Criterion) {
    let mut group = c.benchmark_group("sync_panel/dag");
    for n in SIZES {
        let mut app = common::app(Shape::dags(n));
        common::navigate_to(&mut app, &Panel::Dag);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_function(BenchmarkId::from_parameter(n), |b| {
            b.iter(|| {
                app.sync_panel(black_box(&Panel::Dag));
                black_box(app.dags.table.len())
            });
        });
    }
    group.finish();
}

fn sync_dag_runs(c: &mut Criterion) {
    let mut group = c.benchmark_group("sync_panel/dagrun");
    for n in SIZES {
        let mut app = common::app(Shape {
            dag_runs: n,
            ..Shape::dags(10)
        });
        common::navigate_to(&mut app, &Panel::DAGRun);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_function(BenchmarkId::from_parameter(n), |b| {
            b.iter(|| {
                app.sync_panel(black_box(&Panel::DAGRun));
                black_box(app.dagruns.table.len())
            });
        });
    }
    group.finish();
}

fn sync_task_instances(c: &mut Criterion) {
    let mut group = c.benchmark_group("sync_panel/taskinstance");
    for n in SIZES {
        let mut app = common::app(Shape {
            task_instances: n,
            ..Shape::dags(10)
        });
        common::navigate_to(&mut app, &Panel::TaskInstance);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_function(BenchmarkId::from_parameter(n), |b| {
            b.iter(|| {
                app.sync_panel(black_box(&Panel::TaskInstance));
                black_box(app.task_instances.table.len())
            });
        });
    }
    group.finish();
}

fn allocations(c: &mut Criterion<AllocCount>) {
    let mut group = c.benchmark_group("allocs/sync_panel");
    for n in SIZES {
        let mut app = common::app(Shape {
            dags: n,
            dag_runs: n,
            task_instances: n,
            ..Shape::dags(n)
        });
        for panel in [Panel::Dag, Panel::DAGRun, Panel::TaskInstance] {
            common::navigate_to(&mut app, &panel);
            group.bench_function(BenchmarkId::new(format!("{panel:?}"), n), |b| {
                b.iter(|| app.sync_panel(black_box(&panel)));
            });
        }
    }
    group.finish();
}

criterion_group!(timing, sync_dags, sync_dag_runs, sync_task_instances);
criterion_group! {
    name = allocs;
    config = Criterion::default().with_measurement(AllocCount);
    targets = allocations
}
criterion_main!(timing, allocs);
