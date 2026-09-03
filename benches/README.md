# Benchmarks

Criterion benches for the hot paths of the TUI. They run headlessly with
synthetic data and never touch the network.

| Bench | What it measures |
|---|---|
| `filter` | `filter_items` and `FilterableTable::set_items` at 100 / 1k / 10k DAGs |
| `sync` | `App::sync_panel`, the copy from the environment cache into a panel table |
| `logs` | Ingesting, scrolling and searching log bodies of 64 KiB to 16 MiB |
| `render` | One full frame per panel into ratatui's `TestBackend` |
| `decode` | Deserializing v1 and v2 DAG-list responses |

Groups prefixed `allocs/` report heap allocations per iteration instead of
time, via a counting global allocator installed in every bench binary.

```sh
cargo bench                       # everything
cargo bench --bench render        # one binary
cargo bench -- 'sync_panel/dag'   # one group by name
```

To compare against `main`, save a baseline first and use
[critcmp](https://github.com/BurntSushi/critcmp):

```sh
git switch main && cargo bench -- --save-baseline main
git switch my-branch && cargo bench -- --save-baseline mine
critcmp main mine
```

`cargo clippy --all-targets` compiles the benches, so CI catches breakage
without running them.
