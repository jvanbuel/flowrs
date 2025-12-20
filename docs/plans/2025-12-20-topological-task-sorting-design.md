# Topological Task Sorting Design

## Overview

Sort task instances within a DagRun by their topological order in the DAG graph, providing users with a visual ordering that follows execution flow.

## Problem

Currently, task instances are displayed in an undefined order. Users want to see tasks ordered by their position in the DAG—upstream tasks first, downstream tasks later—to better understand execution flow.

### Challenge: Evolving DAGs

DAG definitions can change between runs. The Airflow REST API only exposes the *current* DAG structure (`/dags/{dag_id}/tasks`), not historical versions. This means:
- Task instances from old runs may reference tasks no longer in the current DAG
- The graph structure may have changed since the DagRun executed

### Solution: Hybrid Approach

Use current DAG structure when possible, fall back to timestamp-based ordering for orphaned tasks:
- Tasks in current DAG: sorted by topological order
- Orphaned tasks (in TaskInstances but not current DAG): sorted by `start_date`, appended after graph-sorted tasks

## Data Model Changes

### New Task Struct

```rust
// src/airflow/model/common/task.rs
pub struct Task {
    pub task_id: String,
    pub downstream_task_ids: Vec<String>,
}

pub struct TaskList {
    pub tasks: Vec<Task>,
}
```

### TaskGraph Helper

```rust
// src/airflow/graph.rs
pub struct TaskGraph {
    sorted_task_ids: Vec<String>,
    task_positions: HashMap<String, usize>,
}

impl TaskGraph {
    pub fn from_tasks(tasks: &[Task]) -> Self {
        // Kahn's algorithm for topological sort
    }

    pub fn position(&self, task_id: &str) -> Option<usize> {
        self.task_positions.get(task_id).copied()
    }
}
```

## API Client Changes

### New Trait

```rust
#[async_trait]
pub trait TaskOperations {
    async fn list_tasks(&self, dag_id: &str) -> Result<TaskList>;
}
```

### Endpoint

```
GET /api/v1/dags/{dag_id}/tasks
```

Response includes `downstream_task_ids` array for each task.

## Sorting Algorithm

```rust
pub fn sort_task_instances(
    instances: &mut [TaskInstance],
    graph: &TaskGraph,
) {
    instances.sort_by(|a, b| {
        match (graph.position(&a.task_id), graph.position(&b.task_id)) {
            (Some(pos_a), Some(pos_b)) => pos_a.cmp(&pos_b),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => a.start_date.cmp(&b.start_date),
        }
    });
}
```

## Worker Integration

### New Message

```rust
pub enum WorkerMessage {
    // ... existing
    UpdateTasks { dag_id: String },
}
```

### Handler

```rust
WorkerMessage::UpdateTasks { dag_id } => {
    match client.list_tasks(&dag_id).await {
        Ok(task_list) => {
            let graph = TaskGraph::from_tasks(&task_list.tasks);
            let mut app = app.lock().unwrap();
            app.task_instances.task_graph = Some(graph);
            app.task_instances.sort_task_instances();
            app.task_instances.filter_task_instances();
        }
        Err(e) => {
            log::warn!("Failed to fetch tasks for {}: {}", dag_id, e);
        }
    }
}
```

### Trigger

Emit `UpdateTasks` when navigating into a DagRun, alongside existing `UpdateTaskInstances`.

## TaskInstanceModel Changes

```rust
pub struct TaskInstanceModel {
    // ... existing fields
    pub task_graph: Option<TaskGraph>,
}

impl TaskInstanceModel {
    pub fn sort_task_instances(&mut self) {
        if let Some(graph) = &self.task_graph {
            sort_task_instances(&mut self.all, graph);
        }
    }

    // filter_task_instances() remains unchanged
}
```

Call order: `sort_task_instances()` then `filter_task_instances()`.

## Edge Cases

| Scenario | Handling |
|----------|----------|
| Task fetch fails | Log warning, keep unsorted order |
| Empty DAG | Empty graph, all tasks sorted by timestamp |
| Cyclic dependencies | Kahn's algorithm detects, fall back to timestamp |
| Tasks arrive before graph | Display unsorted, re-sort when graph arrives |
| DAG changed since run | Orphans sorted by timestamp, appended at end |

## Files to Modify/Create

- `src/airflow/model/common/task.rs` (new)
- `src/airflow/graph.rs` (new)
- `src/airflow/client/v1/task.rs` (new)
- `src/airflow/client/v2/task.rs` (new)
- `src/airflow/traits.rs` - add `TaskOperations`
- `src/app/worker.rs` - add `UpdateTasks` message
- `src/app/model/taskinstances.rs` - add `task_graph` field and sort method
