//! `filter_items` and the table refresh that wraps it.

mod common;

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use flowrs_tui::app::model::filter::{filter_items, FilterCondition};
use flowrs_tui::app::model::filterable_table::FilterableTable;

const SIZES: [usize; 3] = [100, 1_000, 10_000];

fn conditions() -> [(&'static str, Vec<FilterCondition>); 3] {
    [
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

fn set_items(c: &mut Criterion) {
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

criterion_group!(benches, filter_items_bench, set_items);
criterion_main!(benches);
