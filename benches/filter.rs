//! Filtering cost: the raw match loop and the table refresh that wraps it.

mod common;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use flowrs_tui::app::model::filter::{filter_items, FilterCondition};
use flowrs_tui::app::model::filterable_table::FilterableTable;
use std::hint::black_box;

use common::AllocCount;

const SIZES: [usize; 3] = [100, 1_000, 10_000];

fn conditions() -> Vec<(&'static str, Vec<FilterCondition>)> {
    vec![
        ("none", vec![]),
        ("primary", vec![FilterCondition::primary("dag_000")]),
        (
            "owner+tag",
            vec![
                FilterCondition::new("owners", "team-data", false),
                FilterCondition::new("tags", "etl", false),
            ],
        ),
    ]
}

fn filter_items_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_items");
    for n in SIZES {
        let dags = common::dags(n);
        group.throughput(Throughput::Elements(n as u64));
        for (name, conds) in conditions() {
            group.bench_with_input(BenchmarkId::new(name, n), &dags, |b, dags| {
                b.iter(|| filter_items(black_box(dags), black_box(&conds)));
            });
        }
    }
    group.finish();
}

fn table_refresh(c: &mut Criterion) {
    let mut group = c.benchmark_group("filterable_table/set_items");
    for n in SIZES {
        let dags = common::dags(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &dags, |b, dags| {
            let mut table = FilterableTable::new();
            b.iter_batched(
                || dags.clone(),
                |items| {
                    table.set_items(items);
                    black_box(table.len())
                },
                BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

fn allocations(c: &mut Criterion<AllocCount>) {
    let mut group = c.benchmark_group("allocs/filter");
    for n in SIZES {
        let dags = common::dags(n);
        group.bench_with_input(BenchmarkId::new("filter_items", n), &dags, |b, dags| {
            let conds = vec![FilterCondition::primary("dag_000")];
            b.iter(|| filter_items(black_box(dags), black_box(&conds)));
        });
        group.bench_with_input(BenchmarkId::new("set_items", n), &dags, |b, dags| {
            let mut table = FilterableTable::new();
            b.iter_batched(
                || dags.clone(),
                |items| table.set_items(items),
                BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

criterion_group!(timing, filter_items_bench, table_refresh);
criterion_group! {
    name = allocs;
    config = Criterion::default().with_measurement(AllocCount);
    targets = allocations
}
criterion_main!(timing, allocs);
