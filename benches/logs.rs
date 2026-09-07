//! Log panel: ingest, one scroll step, and a search, driven through key events.

mod common;

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use flowrs_tui::app::events::custom::FlowrsEvent;
use flowrs_tui::app::model::logs::LogModel;
use flowrs_tui::app::model::Model;
use flowrs_tui::app::state::NavigationContext;

const SIZES: [usize; 3] = [64 * 1024, 1024 * 1024, 16 * 1024 * 1024];

fn ctx() -> NavigationContext {
    NavigationContext::Task {
        environment: common::ENV.to_string(),
        dag_id: common::dag_id(0),
        dag_run_id: common::dag_run_id(0),
        task_id: common::task_id(0),
        task_try: 1,
    }
}

fn key(code: KeyCode) -> FlowrsEvent {
    FlowrsEvent::Key(KeyEvent::new(code, KeyModifiers::empty()))
}

fn model_with(bytes: usize) -> LogModel {
    let mut model = LogModel::new(10);
    model.update_logs(common::logs(bytes, 1));
    model
}

fn label(bytes: usize) -> String {
    format!("{}KiB", bytes / 1024)
}

fn update_logs(c: &mut Criterion) {
    let mut group = c.benchmark_group("logs/update_logs");
    for bytes in SIZES {
        let logs = common::logs(bytes, 1);
        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_function(BenchmarkId::from_parameter(label(bytes)), |b| {
            let mut model = LogModel::new(10);
            b.iter_batched(
                || logs.clone(),
                |logs| model.update_logs(logs),
                BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

fn scroll_down(c: &mut Criterion) {
    let mut group = c.benchmark_group("logs/scroll_down");
    let ctx = ctx();
    for bytes in SIZES {
        let mut model = model_with(bytes);
        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_function(BenchmarkId::from_parameter(label(bytes)), |b| {
            b.iter(|| black_box(model.update(&key(KeyCode::Down), &ctx)));
        });
    }
    group.finish();
}

fn search(c: &mut Criterion) {
    let mut group = c.benchmark_group("logs/search");
    let ctx = ctx();
    for bytes in SIZES {
        let mut model = model_with(bytes);
        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_function(BenchmarkId::from_parameter(label(bytes)), |b| {
            b.iter(|| {
                model.update(&key(KeyCode::Char('/')), &ctx);
                for ch in "ERROR".chars() {
                    model.update(&key(KeyCode::Char(ch)), &ctx);
                }
                model.update(&key(KeyCode::Enter), &ctx);
                black_box(model.update(&key(KeyCode::Esc), &ctx))
            });
        });
    }
    group.finish();
}

criterion_group!(benches, update_logs, scroll_down, search);
criterion_main!(benches);
