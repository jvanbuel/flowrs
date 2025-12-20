# Topological Task Sorting Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Sort task instances within a DagRun by their topological order in the DAG graph, with timestamp fallback for orphaned tasks.

**Architecture:** Fetch task definitions via `/dags/{dag_id}/tasks` API to get `downstream_task_ids`. Build a directed graph and perform Kahn's algorithm for topological sort. Store sorted order in `TaskInstanceModel` and apply when displaying tasks.

**Tech Stack:** Rust, async_trait, serde, reqwest

---

## Task 1: Create Common Task Model

**Files:**
- Create: `src/airflow/model/common/task.rs`
- Modify: `src/airflow/model/common/mod.rs`

**Step 1: Create the task model file**

```rust
// src/airflow/model/common/task.rs
use crate::airflow::client::v1;
use crate::airflow::client::v2;

/// Common Task model representing a task definition in a DAG
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Task {
    pub task_id: String,
    pub downstream_task_ids: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct TaskList {
    pub tasks: Vec<Task>,
}
```

**Step 2: Add module to mod.rs**

In `src/airflow/model/common/mod.rs`, add:

```rust
pub mod task;

// Add to re-exports:
pub use task::{Task, TaskList};
```

**Step 3: Verify it compiles**

Run: `cargo build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add src/airflow/model/common/task.rs src/airflow/model/common/mod.rs
git commit -m "feat(model): add Task and TaskList common models"
```

---

## Task 2: Create TaskGraph with Topological Sort

**Files:**
- Create: `src/airflow/graph.rs`
- Modify: `src/airflow/mod.rs` (or `src/lib.rs` depending on structure)

**Step 1: Write the unit test for topological sort**

```rust
// src/airflow/graph.rs
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::airflow::model::common::{Task, TaskInstance};

/// Stores topologically sorted task order for O(1) position lookup
#[derive(Default, Debug, Clone)]
pub struct TaskGraph {
    sorted_task_ids: Vec<String>,
    task_positions: HashMap<String, usize>,
}

impl TaskGraph {
    /// Build a TaskGraph from task definitions using Kahn's algorithm
    pub fn from_tasks(tasks: &[Task]) -> Self {
        if tasks.is_empty() {
            return Self::default();
        }

        // Build adjacency list and in-degree map
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();

        // Initialize all tasks with 0 in-degree
        for task in tasks {
            in_degree.entry(&task.task_id).or_insert(0);
            adjacency.entry(&task.task_id).or_default();
        }

        // Build edges from downstream_task_ids
        for task in tasks {
            for downstream_id in &task.downstream_task_ids {
                if let Some(degree) = in_degree.get_mut(downstream_id.as_str()) {
                    *degree += 1;
                }
                adjacency
                    .entry(&task.task_id)
                    .or_default()
                    .push(downstream_id.as_str());
            }
        }

        // Kahn's algorithm
        let mut queue: VecDeque<&str> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut sorted_task_ids = Vec::with_capacity(tasks.len());

        while let Some(task_id) = queue.pop_front() {
            sorted_task_ids.push(task_id.to_string());

            if let Some(neighbors) = adjacency.get(task_id) {
                for &neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }

        // Build position lookup map
        let task_positions: HashMap<String, usize> = sorted_task_ids
            .iter()
            .enumerate()
            .map(|(i, id)| (id.clone(), i))
            .collect();

        Self {
            sorted_task_ids,
            task_positions,
        }
    }

    /// Get the topological position of a task. Returns None if task not in graph.
    pub fn position(&self, task_id: &str) -> Option<usize> {
        self.task_positions.get(task_id).copied()
    }
}

/// Sort task instances by topological order, with timestamp fallback for orphans
pub fn sort_task_instances(instances: &mut [TaskInstance], graph: &TaskGraph) {
    instances.sort_by(|a, b| {
        match (graph.position(&a.task_id), graph.position(&b.task_id)) {
            // Both in graph: use topological order
            (Some(pos_a), Some(pos_b)) => pos_a.cmp(&pos_b),
            // Only a in graph: a comes first
            (Some(_), None) => Ordering::Less,
            // Only b in graph: b comes first
            (None, Some(_)) => Ordering::Greater,
            // Neither in graph: fall back to start_date
            (None, None) => a.start_date.cmp(&b.start_date),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topological_sort_linear_chain() {
        // A -> B -> C
        let tasks = vec![
            Task {
                task_id: "A".to_string(),
                downstream_task_ids: vec!["B".to_string()],
            },
            Task {
                task_id: "B".to_string(),
                downstream_task_ids: vec!["C".to_string()],
            },
            Task {
                task_id: "C".to_string(),
                downstream_task_ids: vec![],
            },
        ];

        let graph = TaskGraph::from_tasks(&tasks);

        assert_eq!(graph.position("A"), Some(0));
        assert_eq!(graph.position("B"), Some(1));
        assert_eq!(graph.position("C"), Some(2));
    }

    #[test]
    fn test_topological_sort_diamond() {
        //     A
        //    / \
        //   B   C
        //    \ /
        //     D
        let tasks = vec![
            Task {
                task_id: "A".to_string(),
                downstream_task_ids: vec!["B".to_string(), "C".to_string()],
            },
            Task {
                task_id: "B".to_string(),
                downstream_task_ids: vec!["D".to_string()],
            },
            Task {
                task_id: "C".to_string(),
                downstream_task_ids: vec!["D".to_string()],
            },
            Task {
                task_id: "D".to_string(),
                downstream_task_ids: vec![],
            },
        ];

        let graph = TaskGraph::from_tasks(&tasks);

        // A must come first, D must come last
        assert_eq!(graph.position("A"), Some(0));
        assert_eq!(graph.position("D"), Some(3));
        // B and C can be in either order (positions 1 and 2)
        assert!(graph.position("B").unwrap() < graph.position("D").unwrap());
        assert!(graph.position("C").unwrap() < graph.position("D").unwrap());
    }

    #[test]
    fn test_topological_sort_empty() {
        let tasks: Vec<Task> = vec![];
        let graph = TaskGraph::from_tasks(&tasks);
        assert_eq!(graph.position("A"), None);
    }

    #[test]
    fn test_unknown_task_returns_none() {
        let tasks = vec![Task {
            task_id: "A".to_string(),
            downstream_task_ids: vec![],
        }];
        let graph = TaskGraph::from_tasks(&tasks);
        assert_eq!(graph.position("unknown"), None);
    }
}
```

**Step 2: Add module to airflow**

Find the airflow module file (likely `src/airflow/mod.rs` or add to `src/lib.rs`) and add:

```rust
pub mod graph;
```

**Step 3: Run the tests**

Run: `cargo test graph`
Expected: All 4 tests pass

**Step 4: Commit**

```bash
git add src/airflow/graph.rs
git commit -m "feat(graph): add TaskGraph with Kahn's topological sort"
```

---

## Task 3: Create TaskOperations Trait

**Files:**
- Create: `src/airflow/traits/task.rs`
- Modify: `src/airflow/traits/mod.rs`

**Step 1: Create the trait file**

```rust
// src/airflow/traits/task.rs
use anyhow::Result;
use async_trait::async_trait;

use crate::airflow::model::common::TaskList;

/// Trait for task definition operations
#[async_trait]
pub trait TaskOperations: Send + Sync {
    /// List all tasks for a DAG
    async fn list_tasks(&self, dag_id: &str) -> Result<TaskList>;
}
```

**Step 2: Add to traits/mod.rs**

In `src/airflow/traits/mod.rs`:

```rust
pub mod task;

pub use task::TaskOperations;
```

**Step 3: Add TaskOperations to AirflowClient super-trait**

In `src/airflow/traits/mod.rs`, modify the `AirflowClient` trait:

```rust
pub trait AirflowClient:
    DagOperations + DagRunOperations + TaskInstanceOperations + LogOperations + DagStatsOperations + TaskOperations
{
    // ... existing methods
}
```

**Step 4: Verify it compiles (expect errors - implementations missing)**

Run: `cargo build`
Expected: Errors about missing `TaskOperations` implementations for V1Client and V2Client

**Step 5: Commit**

```bash
git add src/airflow/traits/task.rs src/airflow/traits/mod.rs
git commit -m "feat(traits): add TaskOperations trait for fetching DAG tasks"
```

---

## Task 4: Implement V1 Task Response Model

**Files:**
- Create: `src/airflow/client/v1/model/task.rs`
- Modify: `src/airflow/client/v1/model/mod.rs`
- Modify: `src/airflow/model/common/task.rs`

**Step 1: Create V1 response model**

```rust
// src/airflow/client/v1/model/task.rs
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskCollectionResponse {
    pub tasks: Vec<TaskResponse>,
    pub total_entries: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskResponse {
    pub task_id: String,
    #[serde(default)]
    pub downstream_task_ids: Vec<String>,
    // Other fields we don't need but may be present
    #[serde(default)]
    pub task_display_name: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
}
```

**Step 2: Add to model/mod.rs**

In `src/airflow/client/v1/model/mod.rs`:

```rust
pub mod task;
```

**Step 3: Add From implementation in common/task.rs**

In `src/airflow/model/common/task.rs`, add:

```rust
use crate::airflow::client::v1;

impl From<v1::model::task::TaskResponse> for Task {
    fn from(value: v1::model::task::TaskResponse) -> Self {
        Task {
            task_id: value.task_id,
            downstream_task_ids: value.downstream_task_ids,
        }
    }
}

impl From<v1::model::task::TaskCollectionResponse> for TaskList {
    fn from(value: v1::model::task::TaskCollectionResponse) -> Self {
        TaskList {
            tasks: value.tasks.into_iter().map(Into::into).collect(),
        }
    }
}
```

**Step 4: Verify it compiles**

Run: `cargo build`
Expected: Build succeeds (still errors for missing trait impl)

**Step 5: Commit**

```bash
git add src/airflow/client/v1/model/task.rs src/airflow/client/v1/model/mod.rs src/airflow/model/common/task.rs
git commit -m "feat(v1): add Task response model for V1 API"
```

---

## Task 5: Implement V1 TaskOperations

**Files:**
- Create: `src/airflow/client/v1/task.rs`
- Modify: `src/airflow/client/v1/mod.rs`

**Step 1: Create V1 implementation**

```rust
// src/airflow/client/v1/task.rs
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Method;

use super::model;
use super::V1Client;
use crate::airflow::{model::common::TaskList, traits::TaskOperations};

#[async_trait]
impl TaskOperations for V1Client {
    async fn list_tasks(&self, dag_id: &str) -> Result<TaskList> {
        let response = self
            .base_api(Method::GET, &format!("dags/{dag_id}/tasks"))?
            .send()
            .await?
            .error_for_status()?;

        let task_collection: model::task::TaskCollectionResponse = response.json().await?;
        Ok(task_collection.into())
    }
}
```

**Step 2: Add module to v1/mod.rs**

In `src/airflow/client/v1/mod.rs`, add:

```rust
mod task;
```

**Step 3: Verify it compiles**

Run: `cargo build`
Expected: Errors only for missing V2 implementation

**Step 4: Commit**

```bash
git add src/airflow/client/v1/task.rs src/airflow/client/v1/mod.rs
git commit -m "feat(v1): implement TaskOperations for V1Client"
```

---

## Task 6: Implement V2 Task Response Model and TaskOperations

**Files:**
- Create: `src/airflow/client/v2/model/task.rs`
- Modify: `src/airflow/client/v2/model/mod.rs`
- Create: `src/airflow/client/v2/task.rs`
- Modify: `src/airflow/client/v2/mod.rs`
- Modify: `src/airflow/model/common/task.rs`

**Step 1: Create V2 response model**

```rust
// src/airflow/client/v2/model/task.rs
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskCollectionResponse {
    pub tasks: Vec<TaskResponse>,
    pub total_entries: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskResponse {
    pub task_id: String,
    #[serde(default)]
    pub downstream_task_ids: Vec<String>,
    #[serde(default)]
    pub task_display_name: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
}
```

**Step 2: Add to v2/model/mod.rs**

```rust
pub mod task;
```

**Step 3: Add From implementations in common/task.rs**

```rust
use crate::airflow::client::v2;

impl From<v2::model::task::TaskResponse> for Task {
    fn from(value: v2::model::task::TaskResponse) -> Self {
        Task {
            task_id: value.task_id,
            downstream_task_ids: value.downstream_task_ids,
        }
    }
}

impl From<v2::model::task::TaskCollectionResponse> for TaskList {
    fn from(value: v2::model::task::TaskCollectionResponse) -> Self {
        TaskList {
            tasks: value.tasks.into_iter().map(Into::into).collect(),
        }
    }
}
```

**Step 4: Create V2 implementation**

```rust
// src/airflow/client/v2/task.rs
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Method;

use super::model;
use super::V2Client;
use crate::airflow::{model::common::TaskList, traits::TaskOperations};

#[async_trait]
impl TaskOperations for V2Client {
    async fn list_tasks(&self, dag_id: &str) -> Result<TaskList> {
        let response = self
            .base_api(Method::GET, &format!("dags/{dag_id}/tasks"))?
            .send()
            .await?
            .error_for_status()?;

        let task_collection: model::task::TaskCollectionResponse = response.json().await?;
        Ok(task_collection.into())
    }
}
```

**Step 5: Add module to v2/mod.rs**

```rust
mod task;
```

**Step 6: Verify full build succeeds**

Run: `cargo build`
Expected: Build succeeds with no errors

**Step 7: Commit**

```bash
git add src/airflow/client/v2/model/task.rs src/airflow/client/v2/model/mod.rs \
        src/airflow/client/v2/task.rs src/airflow/client/v2/mod.rs \
        src/airflow/model/common/task.rs
git commit -m "feat(v2): implement TaskOperations for V2Client"
```

---

## Task 7: Add UpdateTasks WorkerMessage

**Files:**
- Modify: `src/app/worker/mod.rs`
- Create: `src/app/worker/tasks.rs`

**Step 1: Add WorkerMessage variant**

In `src/app/worker/mod.rs`, add to `WorkerMessage` enum:

```rust
UpdateTasks {
    dag_id: String,
},
```

**Step 2: Create tasks worker handler**

```rust
// src/app/worker/tasks.rs
use std::sync::{Arc, Mutex};

use log::debug;

use crate::airflow::graph::TaskGraph;
use crate::airflow::traits::AirflowClient;
use crate::app::state::App;

/// Handle fetching task definitions and building the task graph
pub async fn handle_update_tasks(
    app: &Arc<Mutex<App>>,
    client: &Arc<dyn AirflowClient>,
    dag_id: &str,
) {
    debug!("Fetching tasks for DAG: {dag_id}");

    match client.list_tasks(dag_id).await {
        Ok(task_list) => {
            let graph = TaskGraph::from_tasks(&task_list.tasks);
            debug!("Built task graph with {} tasks", task_list.tasks.len());

            let mut app = app.lock().unwrap();
            app.task_instances.task_graph = Some(graph);
            app.task_instances.sort_task_instances();
            app.task_instances.filter_task_instances();
        }
        Err(e) => {
            // Graceful degradation: log warning but don't show error popup
            log::warn!("Failed to fetch tasks for {dag_id}: {e}");
            // Task instances will remain unsorted
        }
    }
}
```

**Step 3: Add module and handler to worker/mod.rs**

Add import:

```rust
mod tasks;
```

Add match arm in `process_message`:

```rust
WorkerMessage::UpdateTasks { dag_id } => {
    tasks::handle_update_tasks(&app, &client, &dag_id).await;
}
```

**Step 4: Verify it compiles (will fail - task_graph field missing)**

Run: `cargo build`
Expected: Error about missing `task_graph` field on `TaskInstanceModel`

**Step 5: Commit**

```bash
git add src/app/worker/mod.rs src/app/worker/tasks.rs
git commit -m "feat(worker): add UpdateTasks message and handler"
```

---

## Task 8: Add TaskGraph to TaskInstanceModel

**Files:**
- Modify: `src/app/model/taskinstances.rs`

**Step 1: Add task_graph field to TaskInstanceModel**

In `src/app/model/taskinstances.rs`, add import:

```rust
use crate::airflow::graph::{sort_task_instances, TaskGraph};
```

Add field to `TaskInstanceModel` struct:

```rust
pub task_graph: Option<TaskGraph>,
```

**Step 2: Add sort_task_instances method**

```rust
impl TaskInstanceModel {
    /// Sort task instances by topological order (or timestamp fallback)
    pub fn sort_task_instances(&mut self) {
        if let Some(graph) = &self.task_graph {
            sort_task_instances(&mut self.all, graph);
        }
    }

    // ... existing methods
}
```

**Step 3: Verify it compiles**

Run: `cargo build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add src/app/model/taskinstances.rs
git commit -m "feat(taskinstances): add task_graph field and sort method"
```

---

## Task 9: Emit UpdateTasks When Navigating to DagRun

**Files:**
- Modify: `src/app/model/dagruns.rs`

**Step 1: Add UpdateTasks to the Enter key handler**

In `src/app/model/dagruns.rs`, find the `KeyCode::Enter` handler (around line 431-441) and modify to emit both messages:

```rust
KeyCode::Enter => {
    if let (Some(dag_id), Some(dag_run)) = (&self.dag_id, &self.current()) {
        return (
            Some(FlowrsEvent::Key(*key_event)),
            vec![
                WorkerMessage::UpdateTasks {
                    dag_id: dag_id.clone(),
                },
                WorkerMessage::UpdateTaskInstances {
                    dag_id: dag_id.clone(),
                    dag_run_id: dag_run.dag_run_id.clone(),
                    clear: true,
                },
            ],
        );
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo build`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/app/model/dagruns.rs
git commit -m "feat(dagruns): emit UpdateTasks when navigating to task instances"
```

---

## Task 10: Handle UpdateTasks in app.rs State Initialization

**Files:**
- Modify: `src/app.rs`

**Step 1: Add UpdateTasks handling in the message preprocessing**

In `src/app.rs`, find the message handling section (around line 120+) and add a case for `UpdateTasks`:

```rust
WorkerMessage::UpdateTasks { dag_id } => {
    // Clear task graph when switching DAGs
    if app.task_instances.dag_id.as_ref() != Some(dag_id) {
        app.task_instances.task_graph = None;
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo build`
Expected: Build succeeds

**Step 3: Run clippy**

Run: `cargo clippy`
Expected: No errors or warnings

**Step 4: Run all tests**

Run: `cargo test`
Expected: All tests pass

**Step 5: Commit**

```bash
git add src/app.rs
git commit -m "feat(app): handle UpdateTasks message in state initialization"
```

---

## Task 11: Integration Test

**Files:**
- Modify: `tests/v1_api_test.rs`

**Step 1: Add test for list_tasks**

```rust
#[tokio::test]
async fn test_v1_list_tasks() {
    if !should_run_for_api_version("v1") {
        return;
    }

    let client = create_test_client().expect("Failed to create test client");
    let dag_list = client.list_dags().await.expect("Failed to list DAGs");

    if let Some(dag) = dag_list.dags.first() {
        let result = client.list_tasks(&dag.dag_id).await;
        assert!(
            result.is_ok(),
            "Failed to list tasks: {:?}",
            result.err()
        );

        let task_list = result.unwrap();
        // Most DAGs should have at least one task
        // (empty DAGs are unusual but valid)
    }
}
```

**Step 2: Run the integration test (if test environment available)**

Run: `cargo test v1_list_tasks`
Expected: Test passes (or skips if no test environment)

**Step 3: Commit**

```bash
git add tests/v1_api_test.rs
git commit -m "test: add integration test for list_tasks endpoint"
```

---

## Task 12: Manual Testing

**Step 1: Run the application**

Run: `FLOWRS_LOG=debug cargo run`

**Step 2: Navigate to a DAG with multiple tasks**

1. Select a config/environment
2. Navigate to a DAG
3. Select a DagRun
4. Observe task instances are sorted topologically

**Step 3: Check logs for task graph creation**

Look for log messages like:
- "Fetching tasks for DAG: <dag_id>"
- "Built task graph with N tasks"

**Step 4: Final commit**

```bash
git add -A
git commit -m "feat: topological task instance sorting

Adds topological sorting to task instances within a DagRun:
- Fetches task definitions from /dags/{dag_id}/tasks API
- Builds directed graph from downstream_task_ids
- Uses Kahn's algorithm for topological sort
- Falls back to start_date for orphaned tasks

Closes #XXX"
```

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1 | Common Task model | `model/common/task.rs` |
| 2 | TaskGraph with topological sort | `airflow/graph.rs` |
| 3 | TaskOperations trait | `traits/task.rs` |
| 4 | V1 Task response model | `client/v1/model/task.rs` |
| 5 | V1 TaskOperations impl | `client/v1/task.rs` |
| 6 | V2 Task model + impl | `client/v2/model/task.rs`, `client/v2/task.rs` |
| 7 | UpdateTasks WorkerMessage | `worker/mod.rs`, `worker/tasks.rs` |
| 8 | TaskInstanceModel integration | `model/taskinstances.rs` |
| 9 | Emit UpdateTasks on navigation | `model/dagruns.rs` |
| 10 | State initialization handling | `app.rs` |
| 11 | Integration test | `tests/v1_api_test.rs` |
| 12 | Manual testing | - |
