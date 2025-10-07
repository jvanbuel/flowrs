# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Flowrs is a Terminal User Interface (TUI) application for Apache Airflow built with Rust and the ratatui library. It allows users to monitor, inspect, and manage Airflow DAGs from the terminal.

## Build and Development Commands

### Building
- `cargo build`: Build the project in debug mode
- `cargo build --release`: Build optimized release binary
- `make build`: Build release version and copy to `/usr/local/bin/flowrs`

### Running
- `cargo run`: Run the TUI application (equivalent to `flowrs run`)
- `FLOWRS_LOG=debug cargo run`: Run with debug logging enabled
- `make run`: Run with debug logging

### Testing
- `cargo test`: Run all tests
- `cargo test <test_name>`: Run specific test by name
- `cargo test -- --nocapture`: Run tests with output visible

### Linting
- `cargo clippy`: Run Clippy linter

## Configuration

Flowrs stores configuration in `~/.flowrs` (TOML format). The config file is referenced via `CONFIG_FILE` static in `src/main.rs:17`.

Configuration structure:
- `servers`: Array of Airflow server configurations
- `managed_services`: Array of managed service integrations (currently supports Conveyor)
- `active_server`: Name of currently active server

## Architecture

### Core Components

**Event Loop (src/app.rs:19-120)**
The main event loop follows this pattern:
1. Draw UI via `draw_ui()`
2. Wait for events (keyboard input or tick)
3. Route events to active panel's `update()` method
4. Process returned `WorkerMessage`s by sending to worker channel
5. Handle global events (quit, panel navigation)

**Worker System (src/app/worker.rs)**
Async worker runs in a separate tokio task, processes messages from the event loop via mpsc channel:
- Handles all API calls to Airflow
- Updates shared app state via `Arc<Mutex<App>>`
- Messages defined in `WorkerMessage` enum (src/app/worker.rs:20-73)

**Panel Architecture**
Five main panels that implement the `Model` trait (src/app/model.rs:13-15):
- `Config`: Server configuration selection (src/app/model/config.rs)
- `Dag`: DAG listing and filtering (src/app/model/dags.rs)
- `DAGRun`: DAG run instances (src/app/model/dagruns.rs)
- `TaskInstance`: Task instance details (src/app/model/taskinstances.rs)
- `Logs`: Task logs viewer (src/app/model/logs.rs)

Each panel has:
- A model that handles state and logic
- A `StatefulTable<T>` for rendering (src/app/model.rs:18-58)
- An `update()` method that returns `(Option<FlowrsEvent>, Vec<WorkerMessage>)`
- Popup submodules in `src/app/model/popup/` for modal interactions

**Client Architecture (src/airflow/client.rs)**
- `BaseClient` wraps reqwest HTTP client and handles authentication
- Trait-based client system: `V1Client` (Airflow v2, uses /api/v1) and `V2Client` (Airflow v3, uses /api/v2)
- `create_client()` factory function selects appropriate client based on configuration
- Authenticates via `AirflowAuth` enum: Basic, Token (static or command-based), or Conveyor
- Client modules per resource in `src/airflow/client/v1/` and `src/airflow/client/v2/`

**Commands (src/commands/)**
CLI subcommands using clap:
- `run`: Launch TUI (default if no command specified)
- `config add/list/remove/update/enable`: Manage configuration

### Data Flow

1. User presses key → `EventGenerator` produces `FlowrsEvent`
2. Event routed to active panel's `update()` method
3. Panel returns optional fall-through event + `WorkerMessage` vector
4. Worker receives messages, makes API calls, updates `App` state
5. UI re-renders with updated state

### State Management

Shared state via `Arc<Mutex<App>>`:
- UI reads state during `draw_ui()`
- Worker writes state after API responses
- Optimistic updates: Some operations update state before API call completes (e.g., marking dag runs)

## Key Patterns

### Adding a New API Operation

1. Add variant to `WorkerMessage` enum in `src/app/worker.rs`
2. Implement handler in `Worker::process_message()`
3. Add client method in appropriate `src/airflow/client/<resource>.rs`
4. Emit message from panel's `update()` method

### Adding a New Panel

1. Create model in `src/app/model/<name>.rs`
2. Implement `Model` trait with `update()` method
3. Add panel variant to `Panel` enum in `src/app/state.rs`
4. Add UI rendering in `src/ui.rs`
5. Update panel navigation in `App::next_panel()` and `App::previous_panel()`

## Managed Services

Conveyor integration (src/airflow/managed_services/conveyor.rs):
- Automatically discovers Conveyor environments
- Handles authentication via Conveyor client
- Fetches tokens for API access

## Navigation

- `q`: Quit application
- `Ctrl+C` or `Ctrl+D`: Exit
- `Enter` / `Right` / `l`: Move to next panel
- `Esc` / `Left` / `h`: Move to previous panel
- Panel-specific keys defined in popup command modules
