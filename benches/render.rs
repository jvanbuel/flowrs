//! One full `draw_ui` frame per panel into ratatui's `TestBackend`.

mod common;

use std::hint::black_box;
use std::sync::{Arc, Mutex};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use flowrs_tui::app::state::{App, Panel};
use flowrs_tui::ui::draw_ui;
use flowrs_tui::ui::theme::init_theme;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

use common::Shape;

const COLS: u16 = 220;
const ROWS: u16 = 60;
const SIZES: [usize; 3] = [100, 1_000, 10_000];

fn shape(panel: &Panel, n: usize) -> Shape {
    match panel {
        Panel::Config | Panel::Dag => Shape::dags(n),
        Panel::DAGRun => Shape {
            dag_runs: n,
            ..Shape::dags(10)
        },
        Panel::TaskInstance => Shape {
            task_instances: n,
            ..Shape::dags(10)
        },
        Panel::Logs => Shape {
            log_bytes: n * 128,
            ..Shape::dags(10)
        },
    }
}

/// Text the panel must show; guards against measuring an empty frame.
fn expected_text(panel: &Panel) -> &'static str {
    match panel {
        Panel::Config => "bench",
        Panel::Dag => "dag_00000",
        Panel::DAGRun => "scheduled__2026",
        Panel::TaskInstance => "task_0000",
        Panel::Logs => "taskinstance.py",
    }
}

fn terminal_showing(app: &Arc<Mutex<App>>, panel: &Panel) -> Terminal<TestBackend> {
    init_theme(flowrs_config::Theme::Dark);
    let mut terminal = Terminal::new(TestBackend::new(COLS, ROWS)).expect("test backend");
    terminal.draw(|f| draw_ui(f, app)).expect("draw");
    let screen = terminal.backend().to_string();
    assert!(
        screen.contains(expected_text(panel)),
        "{panel:?} panel rendered without its data:\n{screen}"
    );
    terminal
}

fn frame(c: &mut Criterion) {
    for panel in [Panel::Dag, Panel::DAGRun, Panel::TaskInstance, Panel::Logs] {
        let mut group = c.benchmark_group(format!("frame/{panel:?}"));
        for n in SIZES {
            let mut app = common::app(shape(&panel, n));
            common::navigate_to(&mut app, &panel);
            let app = Arc::new(Mutex::new(app));
            let mut terminal = terminal_showing(&app, &panel);
            group.throughput(Throughput::Elements(n as u64));
            group.bench_function(BenchmarkId::from_parameter(n), |b| {
                b.iter(|| {
                    terminal
                        .draw(|f| draw_ui(f, black_box(&app)))
                        .expect("draw");
                });
            });
        }
        group.finish();
    }
}

criterion_group!(benches, frame);
criterion_main!(benches);
