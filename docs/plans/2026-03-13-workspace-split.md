# Workspace Split Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Split the flowrs monolith into a Cargo workspace with 4 crates: `flowrs-config`, `flowrs-airflow-model`, `flowrs-airflow`, and `flowrs-tui`.

**Architecture:** First resolve all cross-boundary dependencies within the monolith (Phase 1), then extract crates one at a time with tests passing after each extraction (Phase 2), and finally add feature gates for managed services (Phase 3).

**Tech Stack:** Rust, Cargo workspaces, feature flags

---

## Dependency graph (target state)

```
flowrs-config              (serde, toml, clap, dirs, strum)
    ↑
flowrs-airflow-model       (chrono/time, serde, async-trait, anyhow)
    ↑
flowrs-airflow             (reqwest, tokio, aws-sdk-mwaa, google-cloud-auth)
    ↑
flowrs-tui                 (ratatui, crossterm, syntect, ansi-to-tui)
```

## Cross-boundary issues discovered during analysis

These must be resolved before any crate extraction:

| Issue | Location | Resolution |
|-------|----------|------------|
| `OpenItem` in `app::worker` referenced by `AirflowClient` trait | `src/airflow/traits/mod.rs:17,38` | Move `OpenItem` to `airflow::model` |
| Domain models have `From<v1/v2>` impls | `src/airflow/model/common/*.rs` | Move `From` impls to `airflow/client/v1/` and `v2/` modules |
| `gantt.rs` imports `AirflowStateColor` + ratatui | `src/airflow/model/common/gantt.rs:3-4,12` | Move `create_bar()` to TUI crate, keep data in model |
| `AirflowAuth` references managed service auth structs | `src/airflow/config/mod.rs:111-113` | Move `MwaaAuth`, `AstronomerAuth`, `ComposerAuth`, `MwaaTokenType` data structs to config module |
| `FlowrsConfig` uses `CONFIG_PATHS` global | `src/airflow/config/mod.rs:19,150,178,300` | Parameterize: pass `&ConfigPaths` to methods |
| `expand_managed_services()` calls into managed service impls | `src/airflow/config/mod.rs:211-290` | Move to standalone function in `airflow/managed_services/` |

---

## Phase 1: Preparatory refactors (resolve cross-boundary deps)

All changes in this phase happen within the existing single-crate structure. Tests must pass after each task.

### Task 1: Move `OpenItem` from `app::worker` to `airflow::model`

**Why:** The `AirflowClient` trait in `airflow::traits` references `OpenItem` from `app::worker`. This is a reverse dependency (airflow → app). Since `OpenItem` is a data enum describing what can be opened in a browser, it belongs with the domain models.

**Files:**
- Modify: `src/airflow/model.rs`
- Modify: `src/airflow/model/common/mod.rs`
- Create: `src/airflow/model/common/open_item.rs`
- Modify: `src/airflow/traits/mod.rs`
- Modify: `src/app/worker/mod.rs`
- Modify: `src/app/worker/browser.rs`

**Steps:**

1. Create `src/airflow/model/common/open_item.rs` with the `OpenItem` enum moved from `src/app/worker/mod.rs:77-99`. It uses `DagId`, `DagRunId`, `TaskId` from the same module.

2. Add `pub mod open_item;` and `pub use open_item::OpenItem;` to `src/airflow/model/common/mod.rs`.

3. In `src/airflow/traits/mod.rs`, change the import from `use crate::app::worker::OpenItem;` to `use crate::airflow::model::common::OpenItem;`.

4. In `src/app/worker/mod.rs`, remove the `OpenItem` enum definition and add `use crate::airflow::model::common::OpenItem;`. Update the `pub enum WorkerMessage` to keep using `OpenItem` via the import.

5. Update `src/app/worker/browser.rs` if it imports `OpenItem` from `super`.

6. Run: `cargo test`

7. Run: `cargo clippy`

8. Commit: `refactor: move OpenItem to airflow::model`

---

### Task 2: Move managed service auth data structs to config module

**Why:** `AirflowAuth` enum in config references `MwaaAuth`, `AstronomerAuth`, `ComposerAuth` from the managed_services module. These are pure serialization structs (serde only). For the config crate to be self-contained, these data types must live alongside `AirflowAuth`.

**Files:**
- Create: `src/airflow/config/managed_auth.rs`
- Modify: `src/airflow/config/mod.rs`
- Modify: `src/airflow/managed_services/mwaa.rs`
- Modify: `src/airflow/managed_services/astronomer.rs`
- Modify: `src/airflow/managed_services/composer/auth.rs`
- Modify: `src/airflow/managed_services/composer/mod.rs`

**Steps:**

1. Create `src/airflow/config/managed_auth.rs` containing the auth data structs extracted from managed_services. These are:

```rust
use serde::{Deserialize, Serialize};

/// MWAA token type - session cookie (Airflow 2.x) or JWT (Airflow 3.x)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MwaaTokenType {
    SessionCookie(String),
    JwtToken(String),
}

/// MWAA authentication data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MwaaAuth {
    pub token: MwaaTokenType,
    pub environment_name: String,
}

/// Astronomer authentication data
pub struct AstronomerAuth {
    pub api_token: String,
}

// Keep the custom Debug impl that masks the token
impl std::fmt::Debug for AstronomerAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AstronomerAuth")
            .field("api_token", &"***")
            .finish()
    }
}

// AstronomerAuth needs Serialize/Deserialize - check original for derives
// and add Clone

/// Composer authentication data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComposerAuth {
    pub project_id: String,
    pub environment_name: String,
}
```

Note: Check the original files carefully for exact derives, trait impls, and any `Clone` derives needed.

2. Add `pub mod managed_auth;` to `src/airflow/config/mod.rs` and update the `AirflowAuth` enum to reference the local types instead of `super::managed_services::*`. Remove the `use super::managed_services::*` imports from config/mod.rs for the auth data types.

3. In `src/airflow/managed_services/mwaa.rs`: remove `MwaaAuth`, `MwaaTokenType` struct definitions and import them from `crate::airflow::config::managed_auth::{MwaaAuth, MwaaTokenType}`.

4. In `src/airflow/managed_services/astronomer.rs`: remove `AstronomerAuth` struct definition and its Debug impl, import from `crate::airflow::config::managed_auth::AstronomerAuth`.

5. In `src/airflow/managed_services/composer/auth.rs`: remove `ComposerAuth` struct, re-export from config. Update `src/airflow/managed_services/composer/mod.rs` to re-export from the new location.

6. Run: `cargo test`

7. Run: `cargo clippy`

8. Commit: `refactor: move managed service auth data structs to config module`

---

### Task 3: Remove `CONFIG_PATHS` global dependency from `FlowrsConfig`

**Why:** `FlowrsConfig::new()` and `FlowrsConfig::from_file()` reference the `CONFIG_PATHS` static from `crate::main`. For the config crate to be standalone, these methods must take `ConfigPaths` as a parameter.

**Files:**
- Modify: `src/airflow/config/mod.rs`
- Modify: `src/commands/run.rs`
- Modify: `src/commands/config/add.rs`
- Modify: `src/commands/config/list.rs`
- Modify: `src/commands/config/remove.rs`
- Modify: `src/commands/config/update.rs`
- Modify: `src/lib.rs`
- Modify: `src/main.rs`

**Steps:**

1. In `src/airflow/config/mod.rs`:
   - Remove `use crate::CONFIG_PATHS;`
   - Change `FlowrsConfig::new()` to `FlowrsConfig::new(config_paths: &ConfigPaths)` (import `ConfigPaths` from `super::config::paths`)
   - Change `FlowrsConfig::from_file(config_path: Option<&PathBuf>)` to `FlowrsConfig::from_file(config_path: Option<&PathBuf>, config_paths: &ConfigPaths)`
   - Change `FlowrsConfig::write_to_file(&mut self)` to `FlowrsConfig::write_to_file(&mut self, config_paths: &ConfigPaths)`
   - Update all internal references from `CONFIG_PATHS.read_path` to `config_paths.read_path` etc.
   - Update `Default` impl if it calls `new()` — may need to remove Default or make it use a sensible default without ConfigPaths

2. Update all call sites. Search the codebase for `FlowrsConfig::new()`, `FlowrsConfig::from_file(`, and `.write_to_file()`. These are in:
   - `src/commands/run.rs` — pass `&CONFIG_PATHS`
   - `src/commands/config/*.rs` — pass `&CONFIG_PATHS`
   - `src/app.rs` or `src/app/state.rs` — if `write_to_file` is called there (line 155 of app.rs: `app.config.write_to_file()`)

3. Update test code in `src/airflow/config/mod.rs` — tests that call `from_file(None)` or `new()` need to construct a `ConfigPaths` or use a test helper.

4. Run: `cargo test`

5. Run: `cargo clippy`

6. Commit: `refactor: parameterize ConfigPaths instead of using global`

---

### Task 4: Move `expand_managed_services()` out of `FlowrsConfig`

**Why:** This method calls into managed service implementations (Conveyor, MWAA, Astronomer, Composer). It belongs in the `flowrs-airflow` crate (which has the managed service deps), not in the config crate.

**Files:**
- Modify: `src/airflow/config/mod.rs`
- Create: `src/airflow/managed_services/expand.rs`
- Modify: `src/airflow/managed_services.rs`
- Modify: `src/commands/run.rs`

**Steps:**

1. Create `src/airflow/managed_services/expand.rs` with a standalone async function:

```rust
use anyhow::Result;
use crate::airflow::config::{FlowrsConfig, AirflowConfig, ManagedService, GccConfig};
// ... other imports from managed_services

/// Expands a FlowrsConfig by resolving managed services and adding their servers.
/// Returns (config, non_fatal_errors).
pub async fn expand_managed_services(mut config: FlowrsConfig) -> Result<(FlowrsConfig, Vec<String>)> {
    // ... move the body from FlowrsConfig::expand_managed_services()
}
```

2. Add `pub mod expand;` to `src/airflow/managed_services.rs` and `pub use expand::expand_managed_services;`.

3. Remove `expand_managed_services()` method from `FlowrsConfig` impl in `src/airflow/config/mod.rs`. Also remove the managed_services imports at the top of that file (`use super::managed_services::*`).

4. Update `src/commands/run.rs` to call the standalone function:
```rust
// Before:
let (config, errors) = FlowrsConfig::from_file(path.as_ref())?
    .expand_managed_services()
    .await?;

// After:
use crate::airflow::managed_services::expand_managed_services;
let config = FlowrsConfig::from_file(path.as_ref(), &CONFIG_PATHS)?;
let (config, errors) = expand_managed_services(config).await?;
```

5. Run: `cargo test`

6. Run: `cargo clippy`

7. Commit: `refactor: extract expand_managed_services as standalone function`

---

### Task 5: Move `From<v1/v2>` impls from domain models to client modules

**Why:** The domain model files (`airflow/model/common/dag.rs`, etc.) import `airflow::client::v1` and `v2` response types to implement `From` conversions. For the model crate to be independent of the client crate, these `From` impls must live in the client crate (which already depends on the model crate). Rust's orphan rule allows this because the response types are local to the client modules.

**Files:**
- Modify: `src/airflow/model/common/dag.rs` — remove `From<v1::*>` and `From<v2::*>` impls
- Modify: `src/airflow/model/common/dagrun.rs` — same
- Modify: `src/airflow/model/common/dagstats.rs` — same
- Modify: `src/airflow/model/common/log.rs` — same
- Modify: `src/airflow/model/common/task.rs` — same
- Modify: `src/airflow/model/common/taskinstance.rs` — same
- Modify: `src/airflow/model/common/gantt.rs` — remove `From<v1/v2 TaskInstanceTryResponse>` impls
- Modify: `src/airflow/client/v1/dag.rs` — add `From` impl here
- Modify: `src/airflow/client/v1/dagrun.rs` — same
- Modify: `src/airflow/client/v1/dagstats.rs` — same
- Modify: `src/airflow/client/v1/log.rs` — same
- Modify: `src/airflow/client/v1/task.rs` — same
- Modify: `src/airflow/client/v1/taskinstance.rs` — same
- Modify: `src/airflow/client/v2/dag.rs` — add `From` impl here
- Modify: `src/airflow/client/v2/dagrun.rs` — same
- Modify: `src/airflow/client/v2/dagstats.rs` — same
- Modify: `src/airflow/client/v2/log.rs` — same
- Modify: `src/airflow/client/v2/task.rs` — same
- Modify: `src/airflow/client/v2/taskinstance.rs` — same

**Steps:**

This is a large mechanical refactor. For each domain model file, do the following pattern:

1. Open `src/airflow/model/common/dag.rs`. Copy the `From<v1::model::dag::DagResponse>` impl and the `From<v1::model::dag::DagCollectionResponse>` impl. Paste them into `src/airflow/client/v1/dag.rs`, updating imports to use `crate::airflow::model::common::{Dag, DagList, Tag}`.

2. Copy the `From<v2::model::dag::Dag>` impl and `From<v2::model::dag::DagList>` impl. Paste into `src/airflow/client/v2/dag.rs`, updating imports similarly.

3. Remove the `From` impls and the `use crate::airflow::client::{v1, v2};` imports from `src/airflow/model/common/dag.rs`.

4. Repeat for: `dagrun.rs`, `dagstats.rs`, `log.rs`, `task.rs`, `taskinstance.rs`, `gantt.rs`.

5. For `gantt.rs` specifically: also handle the `From<v1/v2::model::taskinstance::TaskInstanceTryResponse>` impls — move them to the respective `v1/taskinstance.rs` and `v2/taskinstance.rs` client files.

6. Run: `cargo test`

7. Run: `cargo clippy`

8. Commit: `refactor: move From<v1/v2> impls from domain models to client modules`

**Important:** The `From` impls in the client modules use `impl From<LocalResponseType> for DomainType`. This works because `LocalResponseType` is defined in the client module (orphan rule: at least one type must be local). If you get orphan rule errors, check that the response type is indeed local to the crate where the impl lives.

---

### Task 6: Move `create_bar()` rendering logic from gantt.rs to TUI

**Why:** `gantt.rs` imports `ratatui` types and `AirflowStateColor` from `ui::constants`. The `create_bar()` method is rendering logic, not domain logic. The `GanttData` and `TaskTryGantt` structs should stay in the model crate, but the bar rendering should live in the TUI.

**Files:**
- Modify: `src/airflow/model/common/gantt.rs`
- Create: `src/ui/gantt.rs`
- Modify: `src/ui.rs`
- Find and update all callers of `gantt.create_bar()` (likely in `src/ui.rs` or `src/ui/*.rs`)

**Steps:**

1. Find all callers of `create_bar`:
   ```
   cargo grep "create_bar"
   ```

2. Create `src/ui/gantt.rs` with the `create_bar` function extracted from `GanttData::create_bar()`. It becomes a free function:

```rust
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use crate::airflow::model::common::gantt::GanttData;
use crate::airflow::model::common::TaskId;
use super::constants::AirflowStateColor;

/// Create a Gantt bar Line for a specific task, sized to `width` characters.
pub fn create_gantt_bar(gantt: &GanttData, task_id: &TaskId, width: usize) -> Line<'static> {
    // ... move body from GanttData::create_bar()
    // The ratio() method is private on GanttData, so either:
    // a) Make ratio() pub on GanttData, or
    // b) Move ratio() logic into this function, or
    // c) Add a pub method on GanttData that returns the data needed
}
```

Note: The `ratio()` helper is private. The cleanest approach is to make it `pub` on `GanttData` since it's a pure calculation.

3. Remove `create_bar()` from `GanttData` impl. Make `ratio()` pub. Remove the `ratatui` and `AirflowStateColor` imports from `gantt.rs`.

4. Add `pub mod gantt;` to `src/ui.rs`. Update callers to use `ui::gantt::create_gantt_bar()`.

5. Run: `cargo test`

6. Run: `cargo clippy`

7. Commit: `refactor: move Gantt bar rendering from model to UI layer`

---

## Phase 2: Create workspace and extract crates

### Task 7: Set up Cargo workspace skeleton

**Why:** Create the workspace structure that all crates will live in. The binary crate stays in the root initially; library crates get created empty.

**Files:**
- Create: `crates/flowrs-config/Cargo.toml`
- Create: `crates/flowrs-config/src/lib.rs`
- Create: `crates/flowrs-airflow-model/Cargo.toml`
- Create: `crates/flowrs-airflow-model/src/lib.rs`
- Create: `crates/flowrs-airflow/Cargo.toml`
- Create: `crates/flowrs-airflow/src/lib.rs`
- Modify: `Cargo.toml` (root — becomes workspace root + binary member)

**Steps:**

1. Create directory structure:
```bash
mkdir -p crates/flowrs-config/src
mkdir -p crates/flowrs-airflow-model/src
mkdir -p crates/flowrs-airflow/src
```

2. Modify root `Cargo.toml` to add workspace section:
```toml
[workspace]
members = [
    ".",
    "crates/flowrs-config",
    "crates/flowrs-airflow-model",
    "crates/flowrs-airflow",
]
resolver = "2"

# Move shared dependencies to workspace level
[workspace.dependencies]
anyhow = "1.0.101"
async-trait = "0.1.89"
chrono = "0.4.43"
clap = { version = "^4.5", features = ["derive", "env"] }
dirs = "6.0.0"
log = { version = "0.4.29", features = ["std"] }
reqwest = { version = "0.12.28", features = ["json", "rustls-tls"] }
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.148"
strum = { version = "0.28.0", features = ["derive"] }
time = { version = "0.3.47", features = ["serde", "serde-human-readable", "parsing", "macros"] }
tokio = { version = "1.49.0", features = ["rt-multi-thread", "sync", "macros"] }
toml = "1.0.6"
```

Update the root `[dependencies]` to use `workspace = true` for shared deps.

3. Create minimal `Cargo.toml` for each library crate with appropriate dependencies (can be filled in during extraction tasks). Start with just the package metadata and `lib.rs` containing `// placeholder`.

4. Run: `cargo build` (should still compile — libraries are empty)

5. Run: `cargo test`

6. Commit: `chore: set up Cargo workspace skeleton`

---

### Task 8: Extract `flowrs-config` crate

**Why:** Config is the leaf dependency — no internal deps. Extract it first.

**Files:**
- Move: `src/airflow/config/mod.rs` → `crates/flowrs-config/src/lib.rs`
- Move: `src/airflow/config/paths.rs` → `crates/flowrs-config/src/paths.rs`
- Move: `src/airflow/config/managed_auth.rs` → `crates/flowrs-config/src/managed_auth.rs`
- Modify: `crates/flowrs-config/Cargo.toml`
- Modify: All files that import from `crate::airflow::config::*`

**Steps:**

1. Set up `crates/flowrs-config/Cargo.toml`:
```toml
[package]
name = "flowrs-config"
version = "0.10.1"
edition = "2021"
rust-version = "1.87.0"
description = "Configuration types for flowrs - a TUI for Apache Airflow"
license = "MIT"
repository = "https://github.com/jvanbuel/flowrs"

[dependencies]
anyhow.workspace = true
clap.workspace = true
dirs.workspace = true
log.workspace = true
serde.workspace = true
strum.workspace = true
toml.workspace = true
```

2. Move `src/airflow/config/paths.rs` to `crates/flowrs-config/src/paths.rs`. Update `crate::` references to `crate::` (should already work since paths.rs has no internal deps).

3. Move `src/airflow/config/managed_auth.rs` to `crates/flowrs-config/src/managed_auth.rs`.

4. Move `src/airflow/config/mod.rs` content to `crates/flowrs-config/src/lib.rs`. Key changes:
   - Replace `use crate::airflow::config::` references with local `crate::` paths
   - The module declarations become `pub mod paths;` and `pub mod managed_auth;`
   - Re-export key types at the crate root

5. Add `flowrs-config = { path = "crates/flowrs-config" }` to root `Cargo.toml` dependencies.

6. Remove `src/airflow/config/` directory. Update `src/airflow.rs` to remove `pub mod config;`.

7. Global find-and-replace across the remaining source: `crate::airflow::config::` → `flowrs_config::` (and similar patterns like `use crate::airflow::config::`).

8. Run: `cargo test`

9. Run: `cargo clippy`

10. Commit: `refactor: extract flowrs-config as workspace crate`

---

### Task 9: Extract `flowrs-airflow-model` crate

**Why:** Domain models and traits depend only on config. Extract next.

**Files:**
- Move: `src/airflow/model/` → `crates/flowrs-airflow-model/src/model/`
- Move: `src/airflow/traits/` → `crates/flowrs-airflow-model/src/traits/`
- Move: `src/airflow/graph.rs` → `crates/flowrs-airflow-model/src/graph.rs`
- Modify: `crates/flowrs-airflow-model/Cargo.toml`
- Modify: `crates/flowrs-airflow-model/src/lib.rs`

**Steps:**

1. Set up `crates/flowrs-airflow-model/Cargo.toml`:
```toml
[package]
name = "flowrs-airflow-model"
version = "0.10.1"
edition = "2021"
rust-version = "1.87.0"
description = "Domain models and traits for the Airflow API"
license = "MIT"
repository = "https://github.com/jvanbuel/flowrs"

[dependencies]
flowrs-config = { path = "../flowrs-config" }
anyhow.workspace = true
async-trait.workspace = true
serde.workspace = true
serde_json.workspace = true
time.workspace = true
```

2. Copy `src/airflow/model/` to `crates/flowrs-airflow-model/src/model/`.
   Copy `src/airflow/traits/` to `crates/flowrs-airflow-model/src/traits/`.
   Copy `src/airflow/graph.rs` to `crates/flowrs-airflow-model/src/graph.rs`.

3. Write `crates/flowrs-airflow-model/src/lib.rs`:
```rust
pub mod graph;
pub mod model;
pub mod traits;

// Re-export commonly used types
pub use model::common;
pub use model::newtype_id;
```

4. Update all `crate::airflow::model::` and `crate::airflow::traits::` references within the moved files to use `crate::model::` and `crate::traits::`.

5. Update `crate::airflow::config::` references in traits to `flowrs_config::`.

6. The `AirflowClient` trait in `traits/mod.rs` references `AirflowVersion` from config — update to `flowrs_config::AirflowVersion`.

7. Add `flowrs-airflow-model = { path = "crates/flowrs-airflow-model" }` to root `Cargo.toml`.

8. Remove the moved directories from `src/airflow/`. Update `src/airflow.rs` to remove `pub mod model;`, `pub mod traits;`, `pub mod graph;`.

9. Global find-and-replace in remaining source:
   - `crate::airflow::model::` → `flowrs_airflow_model::model::`
   - `crate::airflow::traits::` → `flowrs_airflow_model::traits::`
   - `crate::airflow::graph::` → `flowrs_airflow_model::graph::`

10. Run: `cargo test`

11. Run: `cargo clippy`

12. Commit: `refactor: extract flowrs-airflow-model as workspace crate`

---

### Task 10: Extract `flowrs-airflow` crate

**Why:** The client implementation, auth providers, and managed services. Depends on config + model.

**Files:**
- Move: `src/airflow/client/` → `crates/flowrs-airflow/src/client/`
- Move: `src/airflow/managed_services/` → `crates/flowrs-airflow/src/managed_services/`
- Modify: `crates/flowrs-airflow/Cargo.toml`
- Modify: `crates/flowrs-airflow/src/lib.rs`
- Remove: `src/airflow.rs` and `src/airflow/` directory

**Steps:**

1. Set up `crates/flowrs-airflow/Cargo.toml`:
```toml
[package]
name = "flowrs-airflow"
version = "0.10.1"
edition = "2021"
rust-version = "1.87.0"
description = "Airflow API client for flowrs"
license = "MIT"
repository = "https://github.com/jvanbuel/flowrs"

[dependencies]
flowrs-config = { path = "../flowrs-config" }
flowrs-airflow-model = { path = "../flowrs-airflow-model" }
anyhow.workspace = true
async-trait.workspace = true
aws-config = "1.8.11"
aws-sdk-mwaa = "1.99.0"
dirs.workspace = true
google-cloud-auth = { version = "1.6.0", default-features = false, features = ["default-rustls-provider"] }
log.workspace = true
reqwest.workspace = true
serde.workspace = true
serde_json.workspace = true
time.workspace = true
tokio.workspace = true

[dev-dependencies]
mockito = "1.7.1"
```

2. Copy `src/airflow/client/` to `crates/flowrs-airflow/src/client/`.
   Copy `src/airflow/managed_services/` to `crates/flowrs-airflow/src/managed_services/`.

3. Write `crates/flowrs-airflow/src/lib.rs`:
```rust
pub mod client;
pub mod managed_services;

pub use client::create_client;
```

4. Update all `crate::airflow::` references within the moved files:
   - `crate::airflow::config::` → `flowrs_config::`
   - `crate::airflow::model::` → `flowrs_airflow_model::model::`
   - `crate::airflow::traits::` → `flowrs_airflow_model::traits::`
   - `crate::airflow::client::` → `crate::client::`
   - `crate::airflow::managed_services::` → `crate::managed_services::`

5. Add `flowrs-airflow = { path = "crates/flowrs-airflow" }` to root `Cargo.toml`.

6. Remove `src/airflow.rs` and `src/airflow/` directory entirely.

7. Update remaining TUI source to import from the new crates:
   - `crate::airflow::client::create_client` → `flowrs_airflow::client::create_client`
   - etc.

8. Remove `pub mod airflow;` from `src/lib.rs` and `src/main.rs`.

9. Run: `cargo test`

10. Run: `cargo clippy`

11. Commit: `refactor: extract flowrs-airflow as workspace crate`

---

### Task 11: Clean up root crate as `flowrs-tui`

**Why:** The root crate now contains only app/, ui/, commands/, main.rs, and lib.rs. Clean up its Cargo.toml to remove dependencies that moved to library crates.

**Files:**
- Modify: `Cargo.toml` (root)
- Modify: `src/lib.rs`
- Modify: `src/main.rs`

**Steps:**

1. Remove dependencies from root `Cargo.toml` that are no longer directly used:
   - `aws-config`, `aws-sdk-mwaa` → moved to flowrs-airflow
   - `google-cloud-auth` → moved to flowrs-airflow
   - Dependencies only used by model/config crates

2. Verify root `Cargo.toml` has these library crate deps:
```toml
flowrs-config = { path = "crates/flowrs-config" }
flowrs-airflow-model = { path = "crates/flowrs-airflow-model" }
flowrs-airflow = { path = "crates/flowrs-airflow" }
```

3. Clean up `src/lib.rs` — should now only export `app`, `ui`, `commands`.

4. Run: `cargo test` (all workspace tests)

5. Run: `cargo clippy --workspace`

6. Commit: `refactor: clean up flowrs-tui dependencies after workspace split`

---

## Phase 3: Feature gates for managed services

### Task 12: Add feature flags to `flowrs-airflow` for managed services

**Why:** Users who only need basic/token auth shouldn't compile AWS SDK, GCP auth, etc.

**Files:**
- Modify: `crates/flowrs-airflow/Cargo.toml`
- Modify: `crates/flowrs-airflow/src/managed_services.rs` (or mod.rs)
- Modify: `crates/flowrs-airflow/src/client/auth/mod.rs`
- Modify: `crates/flowrs-airflow/src/managed_services/expand.rs`
- Modify: `crates/flowrs-config/src/lib.rs` (AirflowAuth enum needs cfg on variants)

**Steps:**

1. In `crates/flowrs-airflow/Cargo.toml`, add features:
```toml
[features]
default = ["conveyor", "mwaa", "astronomer", "composer"]
conveyor = []
mwaa = ["dep:aws-config", "dep:aws-sdk-mwaa"]
astronomer = []
composer = ["dep:google-cloud-auth"]

[dependencies]
aws-config = { version = "1.8.11", optional = true }
aws-sdk-mwaa = { version = "1.99.0", optional = true }
google-cloud-auth = { version = "1.6.0", default-features = false, features = ["default-rustls-provider"], optional = true }
```

2. Gate managed service modules with `#[cfg(feature = "...")]`:
```rust
// managed_services.rs
#[cfg(feature = "astronomer")]
pub mod astronomer;
#[cfg(feature = "composer")]
pub mod composer;
#[cfg(feature = "conveyor")]
pub mod conveyor;
#[cfg(feature = "mwaa")]
pub mod mwaa;
pub mod expand;
```

3. Gate the match arms in `create_auth_provider()` (in `client/auth/mod.rs`):
```rust
#[cfg(feature = "conveyor")]
AirflowAuth::Conveyor => Ok(Box::new(ConveyorAuthProvider)),
#[cfg(not(feature = "conveyor"))]
AirflowAuth::Conveyor => anyhow::bail!("Conveyor support not compiled. Enable the 'conveyor' feature."),
// ... same pattern for mwaa, astronomer, composer
```

4. Gate the match arms in `expand_managed_services()` similarly.

5. In `crates/flowrs-config/Cargo.toml`, add matching features so that `AirflowAuth` enum variants can be conditionally compiled:
```toml
[features]
default = ["conveyor", "mwaa", "astronomer", "composer"]
conveyor = []
mwaa = []
astronomer = []
composer = []
```

6. Gate `AirflowAuth` enum variants and the corresponding auth data structs in `managed_auth.rs`:
```rust
pub enum AirflowAuth {
    Basic(BasicAuth),
    Token(TokenSource),
    #[cfg(feature = "conveyor")]
    Conveyor,
    #[cfg(feature = "mwaa")]
    Mwaa(MwaaAuth),
    #[cfg(feature = "astronomer")]
    Astronomer(AstronomerAuth),
    #[cfg(feature = "composer")]
    Composer(ComposerAuth),
}
```

**Important:** Gating serde enum variants means that configs with disabled variants will fail to deserialize. Consider whether this is acceptable or if you'd prefer to always deserialize all variants but only gate the *implementations*. The simpler approach: keep all `AirflowAuth` variants always present (no cfg on config crate), and only gate the auth provider implementations + managed service discovery in `flowrs-airflow`.

7. In the root `Cargo.toml`, forward features:
```toml
[dependencies]
flowrs-airflow = { path = "crates/flowrs-airflow", features = ["default"] }
```

8. Test with all features: `cargo test --workspace`

9. Test with no optional features: `cargo test --workspace --no-default-features -p flowrs-airflow`

10. Run: `cargo clippy --workspace`

11. Run: `cargo clippy --workspace --no-default-features`

12. Commit: `feat: add feature flags for managed service integrations`

---

## Verification checklist

After all tasks are complete:

- [ ] `cargo build --workspace` succeeds
- [ ] `cargo test --workspace` passes all tests
- [ ] `cargo clippy --workspace` has no warnings
- [ ] `cargo build --workspace --no-default-features` compiles (basic auth only)
- [ ] `cargo build -p flowrs-config` compiles standalone
- [ ] `cargo build -p flowrs-airflow-model` compiles standalone
- [ ] `cargo build -p flowrs-airflow` compiles standalone
- [ ] The `flowrs` binary works end-to-end (`cargo run`)
- [ ] No circular dependencies between crates
