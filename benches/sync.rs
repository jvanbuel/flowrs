//! `App::sync_panel`: the copy from the environment cache into a panel table.

mod common;

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use flowrs_tui::app::state::Panel;

use common::Shape;

const SIZES: [usize; 3] = [100, 1_000, 5_000];

fn sync_panel(c: &mut Criterion) {
    for panel in [Panel::Dag, Panel::DAGRun, Panel::TaskInstance] {
        let mut group = c.benchmark_group(format!("sync_panel/{panel:?}"));
        for n in SIZES {
            let shape = match panel {
                Panel::DAGRun => Shape {
                    dag_runs: n,
                    ..Shape::dags(10)
                },
                Panel::TaskInstance => Shape {
                    task_instances: n,
                    ..Shape::dags(10)
                },
                _ => Shape::dags(n),
            };
            let mut app = common::app(shape);
            common::navigate_to(&mut app, &panel);
            group.throughput(Throughput::Elements(n as u64));
            group.bench_function(BenchmarkId::from_parameter(n), |b| {
                b.iter(|| app.sync_panel(black_box(&panel)));
            });
        }
        group.finish();
    }
}

criterion_group!(benches, sync_panel);
criterion_main!(benches);
