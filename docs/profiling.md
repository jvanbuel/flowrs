# Profiling the Tokio runtime with dial9

Flowrs can record a [dial9](https://crates.io/crates/dial9) trace of its Tokio
runtime: every task poll, wake, worker park/unpark, plus CPU stack samples. It
is behind the opt-in `dial9` cargo feature, so normal builds are unaffected.

## Build

dial9 sees every task only with `tokio_unstable`, and CPU stacks need frame
pointers. The `profiling` profile is a release build that keeps symbols.

```bash
RUSTFLAGS="--cfg tokio_unstable -C force-frame-pointers=yes" \
  cargo build --profile profiling --features dial9
```

## Run against a local Airflow

```bash
docker run -d --name airflow -p 8080:8080 \
  -e AIRFLOW__CORE__LOAD_EXAMPLES=true \
  -e AIRFLOW__DATABASE__SQL_ALCHEMY_CONN=sqlite:////tmp/airflow.db \
  -e AIRFLOW__CORE__SIMPLE_AUTH_MANAGER_USERS=airflow:admin \
  -e AIRFLOW__CORE__SIMPLE_AUTH_MANAGER_PASSWORDS_FILE=/tmp/pw.json \
  --entrypoint /bin/bash apache/airflow:3.1.0 \
  -c "echo '{\"airflow\":\"airflow\"}' > /tmp/pw.json; sleep infinity"
docker exec airflow airflow db migrate
docker exec -d airflow airflow api-server --port 8080
docker exec -d airflow airflow scheduler
docker exec -d airflow airflow dag-processor
```

Point a config file at it with a token from
`curl -X POST localhost:8080/auth/token -H 'content-type: application/json' -d '{"username":"airflow","password":"airflow"}'`
(`version = "V3"`, `[servers.auth.Token] token = "..."`), then run:

```bash
DIAL9_ENABLED=true DIAL9_TRACE_DIR=/tmp/flowrs-traces DIAL9_ROTATION_SECS=600 \
  target/profiling/flowrs run --file flowrs.toml
```

Quit with `q` so the trace is flushed. Each run writes
`/tmp/flowrs-traces/<boot-id>/trace.0.bin.gz`. See the dial9 README for the
other `DIAL9_*` variables.

## Analyze

- Web viewer: `cargo install --locked dial9-viewer --no-default-features`, then
  `dial9 serve --local-dir /tmp/flowrs-traces`.
- Scripted report: the `dial9` crate ships a Node toolkit under
  `skills/dial9-toolkit/scripts/` in its source (in `~/.cargo/registry/src`);
  `node analyze.js /tmp/flowrs-traces/<boot-id>/trace.0.bin.gz` prints long
  polls, scheduling delays, blocking calls and CPU hotspots.

In traces, `src/main.rs` is the UI event loop (drawing happens there),
`src/app/worker/mod.rs` the per-message API tasks, and
`src/app/events/generator.rs` the terminal input and tick task.
