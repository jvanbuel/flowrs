# Benchmarks

Criterion benches over synthetic data. No network access.

| Bench | Measures |
|---|---|
| `filter` | `filter_items` and `FilterableTable::set_items` at 100 / 1k / 10k DAGs |
| `sync` | `App::sync_panel` for the DAG, DAG-run and task-instance panels |
| `logs` | Log ingest, scroll and search at 64 KiB / 1 MiB / 16 MiB |
| `render` | One `draw_ui` frame per panel into ratatui's `TestBackend` |

```sh
cargo bench                      # or: make bench
cargo bench --bench render
cargo bench -- 'sync_panel/Dag'
```

Compare against `main` with [critcmp](https://github.com/BurntSushi/critcmp):

```sh
cargo bench -- --save-baseline main    # on main
cargo bench -- --save-baseline mine    # on your branch
critcmp main mine
```

`cargo clippy --all-targets` compiles the benches, so CI catches breakage.
