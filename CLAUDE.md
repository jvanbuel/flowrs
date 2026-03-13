# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Flowrs is a Terminal User Interface (TUI) application for Apache Airflow built with Rust and the ratatui library. It allows users to monitor, inspect, and manage Airflow DAGs from the terminal.

## Build and Development Commands

### Requirements
- **Minimum Supported Rust Version (MSRV):** 1.87.0

### Building
- `cargo build`: Build the project in debug mode
- `cargo build --release`: Build optimized release binary
- `make build`: Build release version and copy to `/usr/local/bin/flowrs`

### Running
- `cargo run`: Run the TUI application (equivalent to `flowrs run`)
- `FLOWRS_LOG=debug cargo run`: Run with debug logging enabled
- `make run`: Run with debug logging

### Testing
- `cargo test --workspace`: Run all workspace tests
- `cargo test --workspace --lib --bins`: Run unit tests only (matches CI)
- `cargo test <test_name>`: Run specific test by name
- `cargo test -- --nocapture`: Run tests with output visible

### Linting
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: Run Clippy (matches CI)
- `cargo fmt --all --check`: Check formatting (matches CI)

## Workspace Structure

The project is a Cargo workspace with three crates:

```
flowrs-airflow  (self-contained Airflow HTTP client library, zero workspace deps)
      ↑
flowrs-config   (TUI configuration: FlowrsConfig, ConfigPaths, TOML parsing)
      ↑
flowrs-tui      (binary: TUI app, view models, traits, UI, commands)
```

### flowrs-airflow (`crates/flowrs-airflow/`)
Self-contained Airflow API client library. Has no dependencies on other workspace crates.
- `src/auth.rs`: Auth types (`AirflowAuth`, `BasicAuth`, `TokenSource`, `MwaaAuth`, `AstronomerAuth`, `ComposerAuth`)
- `src/config.rs`: Server config types (`AirflowConfig`, `AirflowVersion`, `ManagedService`, `GccConfig`)
- `src/client/`: HTTP client layer
  - `base.rs`: `BaseClient` wrapping reqwest with auth
  - `auth/`: `AuthProvider` trait and implementations (basic, token, managed service providers)
  - `v1/`: Airflow v2 API client (`V1Client`, uses `/api/v1`), with response models in `v1/model/`
  - `v2/`: Airflow v3 API client (`V2Client`, uses `/api/v2`), with response models in `v2/model/`
- `src/managed_services/`: Managed service discovery (Conveyor, MWAA, Astronomer, Cloud Composer), feature-gated
  - `expand.rs`: `expand_managed_services()` takes `ManagedServiceConfig`, returns discovered `AirflowConfig`s

### flowrs-config (`crates/flowrs-config/`)
TUI-specific configuration management. Depends on `flowrs-airflow` for auth/server types (re-exports them).
- `src/lib.rs`: `FlowrsConfig` struct (servers, managed_services, poll_interval, etc.), TOML parsing/writing
- `src/paths.rs`: `ConfigPaths` for XDG-compliant config file resolution
- `src/auth.rs`: Re-exports auth types from `flowrs-airflow`
- `src/server.rs`: Re-exports server config types from `flowrs-airflow`

### flowrs-tui (root crate, `src/`)
The TUI binary. Depends on both `flowrs-airflow` and `flowrs-config`.
- `src/airflow/`: Airflow integration layer (view models, traits, client wrapper)
  - `client.rs`: `FlowrsClient` enum wrapping `V1Client`/`V2Client`, implements all TUI traits, contains From conversions and URL building
  - `model/`: Domain/view model types (`Dag`, `DagRun`, `TaskInstance`, `Log`, `Task`, `DagStatistic`, `GanttData`, `OpenItem`, newtype IDs, duration utils)
  - `traits/`: Async operation traits (`AirflowClient`, `DagOperations`, `DagRunOperations`, `TaskInstanceOperations`, `LogOperations`, `DagStatsOperations`, `TaskOperations`)
  - `graph.rs`: `TaskGraph` for topological sorting of task instances
- `src/app.rs`: Main event loop
- `src/app/worker/`: Async worker processing `WorkerMessage`s via mpsc channel
- `src/app/model/`: Panel models implementing the `Model` trait
- `src/app/model/popup/`: Modal popup interactions
- `src/ui/`: UI rendering (ratatui widgets, gantt charts, theming)
- `src/commands/`: CLI subcommands (run, config add/list/remove/update/enable)

## Configuration

Flowrs stores configuration in TOML format, following the XDG Base Directory Specification:
- **Primary (XDG):** `$XDG_CONFIG_HOME/flowrs/config.toml` (defaults to `~/.config/flowrs/config.toml`)
- **Legacy fallback:** `~/.flowrs` (read if XDG path doesn't exist)

Config paths are managed via `CONFIG_PATHS` static in `src/main.rs`, which uses `ConfigPaths` from `crates/flowrs-config/src/paths.rs`. Writes always go to the XDG path; reads check XDG first, then legacy. A warning popup is shown if both files exist.

Configuration structure:
- `servers`: Array of Airflow server configurations
- `managed_services`: Array of managed service integrations (Conveyor, MWAA, Astronomer, GCC)
- `active_server`: Name of currently active server
- `poll_interval_ms`: API poll interval in milliseconds (default 2000, minimum 500)

## Architecture

### Event Loop (src/app.rs)
1. Draw UI via `draw_ui()`
2. Wait for events (keyboard input or tick)
3. Route events to active panel's `update()` method
4. Process returned `WorkerMessage`s by sending to worker channel
5. Handle global events (quit, panel navigation)

### Worker System (src/app/worker/)
Async worker runs in a separate tokio task, processes messages from the event loop via mpsc channel:
- Handles all API calls to Airflow via `FlowrsClient`
- Updates shared app state via `Arc<Mutex<App>>`
- Messages defined in `WorkerMessage` enum

### Panel Architecture
Five main panels implement the `Model` trait (src/app/model.rs):
- `Config`: Server configuration selection
- `Dag`: DAG listing and filtering
- `DAGRun`: DAG run instances
- `TaskInstance`: Task instance details
- `Logs`: Task logs viewer

Each panel has:
- A model that handles state and logic
- A `StatefulTable<T>` for rendering
- An `update()` method that returns `(Option<FlowrsEvent>, Vec<WorkerMessage>)`
- Popup submodules in `src/app/model/popup/` for modal interactions

### Client Architecture
- `flowrs-airflow` provides raw HTTP clients (`V1Client`, `V2Client`) returning API response types
- `FlowrsClient` (in `src/airflow/client.rs`) wraps these and implements TUI operation traits
- From impls in `FlowrsClient` convert API response types to TUI view models
- Auth providers handle Basic, Token, Conveyor, MWAA, Astronomer, and Composer authentication

### Data Flow
1. User presses key → `EventGenerator` produces `FlowrsEvent`
2. Event routed to active panel's `update()` method
3. Panel returns optional fall-through event + `WorkerMessage` vector
4. Worker receives messages, calls methods on `FlowrsClient`, updates `App` state
5. UI re-renders with updated state

### State Management
Shared state via `Arc<Mutex<App>>`:
- UI reads state during `draw_ui()`
- Worker writes state after API responses
- Optimistic updates: Some operations update state before API call completes (e.g., marking dag runs)

## Key Patterns

### Adding a New API Operation
1. Add raw HTTP method to `V1Client`/`V2Client` in `crates/flowrs-airflow/src/client/v{1,2}/`
2. Add response model types if needed in `v{1,2}/model/`
3. Add/update the operation trait in `src/airflow/traits/`
4. Implement the trait method in `FlowrsClient` (`src/airflow/client.rs`) with From conversion
5. Add variant to `WorkerMessage` enum in `src/app/worker/`
6. Implement handler in worker
7. Emit message from panel's `update()` method

### Adding a New Panel
1. Create model in `src/app/model/<name>.rs`
2. Implement `Model` trait with `update()` method
3. Add panel variant to `Panel` enum in `src/app/state.rs`
4. Add UI rendering in `src/ui.rs`
5. Update panel navigation in `App::next_panel()` and `App::previous_panel()`

## Managed Services

Feature-gated integrations in `crates/flowrs-airflow/src/managed_services/`:
- **Conveyor** (`conveyor` feature): Discovers environments via Conveyor CLI
- **MWAA** (`mwaa` feature): AWS MWAA via aws-sdk-mwaa
- **Astronomer** (`astronomer` feature): Astronomer API via `ASTRO_API_TOKEN`
- **Cloud Composer** (`composer` feature): GCP Composer via google-cloud-auth

All features are enabled by default. Each returns `Vec<AirflowConfig>` with pre-configured auth.

## Navigation

- `q`: Quit application
- `Ctrl+C` or `Ctrl+D`: Exit
- `Enter` / `Right` / `l`: Move to next panel
- `Esc` / `Left` / `h`: Move to previous panel
- Panel-specific keys defined in popup command modules
