//! Topological layout of a DAG's task definitions.
//!
//! The graph is stored as a set of flat, index-aligned tables rather than as
//! `HashMap<String, _>`s keyed by task ID. Every task gets one dense index,
//! and that index is the only thing the other tables are keyed by:
//!
//! - `ids[i]` is the task ID, `level_of[i]` its topological level;
//! - `level_starts` marks where each level begins in `ids`, so "tasks at
//!   level `l`" is a slice, not a scan;
//! - downstream edges are stored in compressed-sparse-row form
//!   (`edge_starts` + `edges`), so an adjacency list is also a slice.
//!
//! Tasks are laid out in display order (level, then ID), which means the dense
//! index doubles as the sort rank used to order task instances. The single
//! `HashMap` that remains is the `id -> index` lookup for names that arrive
//! from outside the graph (task instances, popups). Everything hot goes
//! through indices.

use std::collections::HashMap;
use std::ops::Range;

use super::model::common::{Task, TaskInstance};

/// Stores topological levels for each task, where tasks at the same level
/// can execute in parallel (have the same dependency depth).
///
/// Tasks that are part of a cycle never receive a level and are excluded from
/// the graph entirely (they are unreachable by Kahn's algorithm).
#[derive(Default, Debug, Clone)]
pub struct TaskGraph {
    /// Task IDs in display order: by level, then alphabetically.
    /// A task's position here is its dense index.
    ids: Vec<String>,
    /// `level_of[i]` is the topological level of `ids[i]`.
    level_of: Vec<usize>,
    /// `level_starts[l]..level_starts[l + 1]` is the index range of level `l`.
    /// Has `max_level + 2` entries when the graph is non-empty, otherwise none.
    level_starts: Vec<usize>,
    /// CSR row offsets: the downstream indices of task `i` are
    /// `edges[edge_starts[i]..edge_starts[i + 1]]`.
    edge_starts: Vec<usize>,
    /// CSR column indices (dense task indices), in definition order.
    edges: Vec<usize>,
    /// Lookup from task ID to dense index for callers that only hold a name.
    index: HashMap<String, usize>,
}

impl TaskGraph {
    /// Build a `TaskGraph` from task definitions using level-based Kahn's algorithm.
    /// Tasks at the same dependency depth get the same level.
    #[must_use]
    pub fn from_tasks(tasks: &[Task]) -> Self {
        if tasks.is_empty() {
            return Self::default();
        }

        // Provisional indices follow definition order; edges are resolved to
        // them once here so the rest of the build never touches a string.
        let provisional: HashMap<&str, usize> = tasks
            .iter()
            .enumerate()
            .map(|(i, t)| (t.task_id.as_str(), i))
            .collect();

        let mut in_degree = vec![0usize; tasks.len()];
        let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); tasks.len()];
        for (from, task) in tasks.iter().enumerate() {
            for downstream in &task.downstream_task_ids {
                if let Some(&to) = provisional.get(downstream.as_str()) {
                    in_degree[to] += 1;
                    adjacency[from].push(to);
                }
            }
        }

        // Level-based Kahn's algorithm over the provisional indices.
        let mut level_of_provisional: Vec<Option<usize>> = vec![None; tasks.len()];
        let mut current: Vec<usize> = (0..tasks.len()).filter(|&i| in_degree[i] == 0).collect();
        let mut level = 0;
        while !current.is_empty() {
            let mut next = Vec::new();
            for &i in &current {
                level_of_provisional[i] = Some(level);
                for &to in &adjacency[i] {
                    in_degree[to] -= 1;
                    if in_degree[to] == 0 {
                        next.push(to);
                    }
                }
            }
            current = next;
            level += 1;
        }

        // Display order: level, then task ID. This becomes the dense index.
        let mut order: Vec<(usize, usize)> = level_of_provisional
            .iter()
            .enumerate()
            .filter_map(|(i, l)| l.map(|l| (l, i)))
            .collect();
        order.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| tasks[a.1].task_id.cmp(&tasks[b.1].task_id))
        });

        let mut dense_of_provisional = vec![usize::MAX; tasks.len()];
        for (dense, &(_, provisional)) in order.iter().enumerate() {
            dense_of_provisional[provisional] = dense;
        }

        let mut ids = Vec::with_capacity(order.len());
        let mut level_of = Vec::with_capacity(order.len());
        let mut level_starts = Vec::new();
        let mut edge_starts = Vec::with_capacity(order.len() + 1);
        let mut edges = Vec::new();
        let mut index = HashMap::with_capacity(order.len());

        for (dense, &(level, provisional)) in order.iter().enumerate() {
            while level_starts.len() <= level {
                level_starts.push(dense);
            }
            let task = &tasks[provisional];
            ids.push(task.task_id.clone());
            level_of.push(level);
            index.insert(task.task_id.clone(), dense);
            edge_starts.push(edges.len());
            edges.extend(
                adjacency[provisional]
                    .iter()
                    .map(|&to| dense_of_provisional[to])
                    .filter(|&to| to != usize::MAX),
            );
        }
        if !order.is_empty() {
            level_starts.push(order.len());
        }
        edge_starts.push(edges.len());

        Self {
            ids,
            level_of,
            level_starts,
            edge_starts,
            edges,
            index,
        }
    }

    /// Number of tasks in the graph.
    #[must_use]
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Returns true if the graph contains no tasks.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// Task IDs in display order (level, then alphabetical). The position of
    /// an ID in this slice is its dense index.
    #[must_use]
    pub fn ids(&self) -> &[String] {
        &self.ids
    }

    /// Dense index of a task, which is also its rank in display order.
    /// Returns None if the task is not in the graph.
    #[must_use]
    pub fn index_of(&self, task_id: &str) -> Option<usize> {
        self.index.get(task_id).copied()
    }

    /// Get the topological level of a task. Returns None if task not in graph.
    #[must_use]
    pub fn level(&self, task_id: &str) -> Option<usize> {
        self.index_of(task_id).map(|i| self.level_of[i])
    }

    /// Topological level of the task at a dense index.
    #[must_use]
    pub fn level_at(&self, index: usize) -> usize {
        self.level_of[index]
    }

    /// Get the maximum topological level in the graph (0 when empty).
    #[must_use]
    pub fn max_level(&self) -> usize {
        self.level_starts.len().saturating_sub(2)
    }

    /// Dense index range of all tasks at a level; empty for levels past the end.
    #[must_use]
    pub fn level_range(&self, level: usize) -> Range<usize> {
        match (
            self.level_starts.get(level),
            self.level_starts.get(level + 1),
        ) {
            (Some(&start), Some(&end)) => start..end,
            _ => 0..0,
        }
    }

    /// All task IDs at a given level, sorted alphabetically.
    #[must_use]
    pub fn tasks_at_level(&self, level: usize) -> &[String] {
        &self.ids[self.level_range(level)]
    }

    /// Dense indices of the tasks directly downstream of the task at `index`.
    #[must_use]
    pub fn downstream(&self, index: usize) -> &[usize] {
        &self.edges[self.edge_starts[index]..self.edge_starts[index + 1]]
    }
}

/// Sort task instances by topological level, then alphabetically within each level.
/// Orphans (not in graph) are sorted alphabetically and appended at the end.
///
/// The graph's dense index already encodes `(level, task_id)` order, so each
/// instance needs exactly one lookup to obtain its rank. Ranks are computed
/// once per element and the sort permutes those keys rather than comparing
/// (and re-hashing) the instances themselves.
pub fn sort_task_instances(instances: &mut [TaskInstance], graph: &TaskGraph) {
    // Orphans get the maximum rank and carry their ID so they order among
    // themselves; graph members never allocate.
    instances.sort_by_cached_key(|ti| match graph.index_of(&ti.task_id) {
        Some(rank) => (rank, None),
        None => (usize::MAX, Some(ti.task_id.clone())),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_task_instance(task_id: &str) -> TaskInstance {
        TaskInstance {
            task_id: task_id.into(),
            ..Default::default()
        }
    }

    fn task(id: &str, downstream: &[&str]) -> Task {
        Task {
            task_id: id.to_string(),
            downstream_task_ids: downstream.iter().map(ToString::to_string).collect(),
        }
    }

    #[test]
    fn test_topological_levels_linear_chain() {
        // A -> B -> C
        let tasks = vec![task("A", &["B"]), task("B", &["C"]), task("C", &[])];

        let graph = TaskGraph::from_tasks(&tasks);

        // Each task is at a different level in a linear chain
        assert_eq!(graph.level("A"), Some(0));
        assert_eq!(graph.level("B"), Some(1));
        assert_eq!(graph.level("C"), Some(2));
        assert_eq!(graph.max_level(), 2);
    }

    #[test]
    fn test_topological_levels_diamond() {
        //     A
        //    / \
        //   B   C
        //    \ /
        //     D
        let tasks = vec![
            task("A", &["B", "C"]),
            task("B", &["D"]),
            task("C", &["D"]),
            task("D", &[]),
        ];

        let graph = TaskGraph::from_tasks(&tasks);

        // A is at level 0, B and C are at level 1, D is at level 2
        assert_eq!(graph.level("A"), Some(0));
        assert_eq!(graph.level("B"), Some(1));
        assert_eq!(graph.level("C"), Some(1)); // Same level as B!
        assert_eq!(graph.level("D"), Some(2));
    }

    #[test]
    fn test_topological_levels_empty() {
        let tasks: Vec<Task> = vec![];
        let graph = TaskGraph::from_tasks(&tasks);
        assert_eq!(graph.level("A"), None);
        assert!(graph.is_empty());
        assert_eq!(graph.max_level(), 0);
        assert!(graph.tasks_at_level(0).is_empty());
    }

    #[test]
    fn test_unknown_task_returns_none() {
        let tasks = vec![task("A", &[])];
        let graph = TaskGraph::from_tasks(&tasks);
        assert_eq!(graph.level("unknown"), None);
        assert_eq!(graph.index_of("unknown"), None);
    }

    #[test]
    fn test_ids_are_in_level_then_alphabetical_order() {
        // Definition order is deliberately scrambled.
        let tasks = vec![
            task("D", &[]),
            task("B", &["D"]),
            task("A", &["C", "B"]),
            task("C", &["D"]),
        ];
        let graph = TaskGraph::from_tasks(&tasks);

        assert_eq!(graph.ids(), ["A", "B", "C", "D"]);
        assert_eq!(graph.tasks_at_level(0), ["A"]);
        assert_eq!(graph.tasks_at_level(1), ["B", "C"]);
        assert_eq!(graph.tasks_at_level(2), ["D"]);
        assert_eq!(graph.level_range(1), 1..3);
        assert!(graph.tasks_at_level(3).is_empty());
        for (i, id) in graph.ids().iter().enumerate() {
            assert_eq!(graph.index_of(id), Some(i));
            assert_eq!(graph.level_at(i), graph.level(id).unwrap());
        }
    }

    #[test]
    fn test_downstream_uses_dense_indices_and_drops_unknown_targets() {
        let tasks = vec![
            task("A", &["C", "B", "ghost"]),
            task("B", &[]),
            task("C", &[]),
        ];
        let graph = TaskGraph::from_tasks(&tasks);

        let a = graph.index_of("A").unwrap();
        let b = graph.index_of("B").unwrap();
        let c = graph.index_of("C").unwrap();
        // Definition order of the edges is preserved; "ghost" is not a task.
        assert_eq!(graph.downstream(a), [c, b]);
        assert!(graph.downstream(b).is_empty());
        assert!(graph.downstream(c).is_empty());
    }

    #[test]
    fn test_cycle_members_are_excluded() {
        // A -> B -> C -> B is a cycle hanging off A; only A can be levelled.
        let tasks = vec![task("A", &["B"]), task("B", &["C"]), task("C", &["B"])];
        let graph = TaskGraph::from_tasks(&tasks);

        assert_eq!(graph.len(), 1);
        assert_eq!(graph.level("A"), Some(0));
        assert_eq!(graph.level("B"), None);
        assert_eq!(graph.level("C"), None);
        // A's edge to B points at a task outside the graph, so it is dropped.
        assert!(graph.downstream(0).is_empty());
    }

    #[test]
    fn test_sort_within_level_alphabetically() {
        //     A
        //    /|\
        //   D B C  (all at level 1)
        let tasks = vec![
            task("A", &["D", "B", "C"]),
            task("B", &[]),
            task("C", &[]),
            task("D", &[]),
        ];

        let graph = TaskGraph::from_tasks(&tasks);

        // All at level 1 -> should sort alphabetically
        let mut instances = vec![
            make_task_instance("D"),
            make_task_instance("B"),
            make_task_instance("C"),
        ];

        sort_task_instances(&mut instances, &graph);

        assert_eq!(instances[0].task_id, "B");
        assert_eq!(instances[1].task_id, "C");
        assert_eq!(instances[2].task_id, "D");
    }

    #[test]
    fn test_sort_preserves_level_order() {
        // A -> B -> C (linear chain)
        let tasks = vec![task("A", &["B"]), task("B", &["C"]), task("C", &[])];

        let graph = TaskGraph::from_tasks(&tasks);

        // Instances in wrong order
        let mut instances = vec![
            make_task_instance("C"),
            make_task_instance("A"),
            make_task_instance("B"),
        ];

        sort_task_instances(&mut instances, &graph);

        // Level order is preserved: A (level 0) -> B (level 1) -> C (level 2)
        assert_eq!(instances[0].task_id, "A");
        assert_eq!(instances[1].task_id, "B");
        assert_eq!(instances[2].task_id, "C");
    }

    #[test]
    fn test_sort_appends_orphans_alphabetically_and_keeps_mapped_order() {
        let tasks = vec![task("A", &["B"]), task("B", &[])];
        let graph = TaskGraph::from_tasks(&tasks);

        let mut b_second = make_task_instance("B");
        b_second.map_index = 1;
        let mut instances = vec![
            make_task_instance("zeta"),
            b_second,
            make_task_instance("alpha"),
            make_task_instance("B"),
            make_task_instance("A"),
        ];

        sort_task_instances(&mut instances, &graph);

        let ids: Vec<&str> = instances.iter().map(|ti| &*ti.task_id).collect();
        assert_eq!(ids, ["A", "B", "B", "alpha", "zeta"]);
        // Equal ranks (mapped instances of B) keep their input order.
        assert_eq!(instances[1].map_index, 1);
        assert_eq!(instances[2].map_index, 0);
    }
}
