use std::cmp::Ordering;
use std::collections::HashMap;

use crate::airflow::model::common::{Task, TaskInstance};

/// Stores topological levels for each task, where tasks at the same level
/// can execute in parallel (have the same dependency depth)
#[derive(Default, Debug, Clone)]
pub struct TaskGraph {
    task_levels: HashMap<String, usize>,
}

impl TaskGraph {
    /// Build a `TaskGraph` from task definitions using level-based Kahn's algorithm.
    /// Tasks at the same dependency depth get the same level.
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

        // Level-based Kahn's algorithm
        // All tasks with in-degree 0 start at level 0
        let mut current_level: Vec<&str> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut task_levels: HashMap<String, usize> = HashMap::new();
        let mut level = 0;

        while !current_level.is_empty() {
            let mut next_level: Vec<&str> = Vec::new();

            for task_id in &current_level {
                task_levels.insert((*task_id).to_string(), level);

                if let Some(neighbors) = adjacency.get(*task_id) {
                    for &neighbor in neighbors {
                        if let Some(degree) = in_degree.get_mut(neighbor) {
                            *degree -= 1;
                            if *degree == 0 {
                                next_level.push(neighbor);
                            }
                        }
                    }
                }
            }

            current_level = next_level;
            level += 1;
        }

        Self { task_levels }
    }

    /// Get the topological level of a task. Returns None if task not in graph.
    pub fn level(&self, task_id: &str) -> Option<usize> {
        self.task_levels.get(task_id).copied()
    }
}

/// Sort task instances by topological level, then alphabetically within each level.
/// Orphans (not in graph) are sorted alphabetically and appended at the end.
pub fn sort_task_instances(instances: &mut [TaskInstance], graph: &TaskGraph) {
    instances.sort_by(
        |a, b| match (graph.level(&a.task_id), graph.level(&b.task_id)) {
            (Some(level_a), Some(level_b)) => level_a
                .cmp(&level_b)
                .then_with(|| a.task_id.cmp(&b.task_id)),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => a.task_id.cmp(&b.task_id),
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_task_instance(task_id: &str) -> TaskInstance {
        TaskInstance {
            task_id: task_id.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_topological_levels_linear_chain() {
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

        // Each task is at a different level in a linear chain
        assert_eq!(graph.level("A"), Some(0));
        assert_eq!(graph.level("B"), Some(1));
        assert_eq!(graph.level("C"), Some(2));
    }

    #[test]
    fn test_topological_levels_diamond() {
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
    }

    #[test]
    fn test_unknown_task_returns_none() {
        let tasks = vec![Task {
            task_id: "A".to_string(),
            downstream_task_ids: vec![],
        }];
        let graph = TaskGraph::from_tasks(&tasks);
        assert_eq!(graph.level("unknown"), None);
    }

    #[test]
    fn test_sort_within_level_alphabetically() {
        //     A
        //    /|\
        //   D B C  (all at level 1)
        let tasks = vec![
            Task {
                task_id: "A".to_string(),
                downstream_task_ids: vec!["D".to_string(), "B".to_string(), "C".to_string()],
            },
            Task {
                task_id: "B".to_string(),
                downstream_task_ids: vec![],
            },
            Task {
                task_id: "C".to_string(),
                downstream_task_ids: vec![],
            },
            Task {
                task_id: "D".to_string(),
                downstream_task_ids: vec![],
            },
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
}
