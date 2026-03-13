use std::collections::HashMap;

use time::OffsetDateTime;

use super::taskinstance::TaskInstanceState;
use super::TaskId;

/// A single try/attempt of a task instance, with the fields needed for Gantt chart rendering.
#[derive(Debug, Clone)]
pub struct TaskTryGantt {
    pub try_number: u32,
    pub start_date: Option<OffsetDateTime>,
    pub end_date: Option<OffsetDateTime>,
    pub state: Option<TaskInstanceState>,
}

/// Gantt chart data for all task instances in the current DAG run.
/// Rebuilt each time the task instance list is refreshed.
#[derive(Debug, Clone, Default)]
pub struct GanttData {
    /// The start of the visible time window
    pub window_start: Option<OffsetDateTime>,
    /// The end of the visible time window (`now()` if any task is still running)
    pub window_end: Option<OffsetDateTime>,
    /// Per-task list of tries, keyed by `TaskId`, sorted by `try_number` ascending
    pub task_tries: HashMap<TaskId, Vec<TaskTryGantt>>,
}

impl GanttData {
    /// Build Gantt data from the current task instance list.
    /// Each task instance contributes one segment (its current/latest try).
    ///
    /// Note: tasks are keyed by [`TaskId`] only — `map_index` is not distinguished,
    /// so mapped task tries (e.g. from `expand_task`) are merged into a single bar
    /// whose `char_colors` overwrite each other in the Gantt bar renderer. This is
    /// intentional for now; callers who need per-map-index bars should pre-group
    /// or provide distinct `TaskId` values.
    #[must_use]
    pub fn from_task_instances(tasks: &[super::TaskInstance]) -> Self {
        if tasks.is_empty() {
            return Self::default();
        }

        let mut task_tries: HashMap<TaskId, Vec<TaskTryGantt>> = HashMap::new();

        for task in tasks {
            let entry = task_tries.entry(task.task_id.clone()).or_default();
            entry.push(TaskTryGantt {
                try_number: task.try_number,
                start_date: task.start_date,
                end_date: task.end_date,
                state: task.state.clone(),
            });
        }

        // Sort each task's tries by try_number
        for tries in task_tries.values_mut() {
            tries.sort_by_key(|t| t.try_number);
        }

        let mut gantt = Self {
            window_start: None,
            window_end: None,
            task_tries,
        };
        gantt.recompute_window();
        gantt
    }

    /// Replace the tries for a specific task (called when tries API response arrives).
    pub fn update_tries(&mut self, task_id: &TaskId, mut tries: Vec<TaskTryGantt>) {
        tries.sort_by_key(|t| t.try_number);
        self.task_tries.insert(task_id.clone(), tries);
        self.recompute_window();
    }

    /// Recalculate `window_start` and `window_end` from all tries.
    pub fn recompute_window(&mut self) {
        let mut min_start: Option<OffsetDateTime> = None;
        let mut max_end: Option<OffsetDateTime> = None;
        let mut any_running = false;

        for tries in self.task_tries.values() {
            for t in tries {
                if let Some(start) = t.start_date {
                    min_start = Some(min_start.map_or(start, |cur| cur.min(start)));
                }
                if let Some(end) = t.end_date {
                    max_end = Some(max_end.map_or(end, |cur| cur.max(end)));
                }
                if matches!(
                    t.state,
                    Some(
                        TaskInstanceState::Running
                            | TaskInstanceState::Queued
                            | TaskInstanceState::Scheduled
                    )
                ) {
                    any_running = true;
                }
            }
        }

        self.window_start = min_start;
        self.window_end = if any_running {
            Some(OffsetDateTime::now_utc())
        } else {
            max_end
        };
    }

    /// Calculate the ratio (0.0 to 1.0) for a given timestamp within the window.
    #[must_use]
    pub fn ratio(&self, time: OffsetDateTime) -> f64 {
        match (self.window_start, self.window_end) {
            (Some(start), Some(end)) => {
                let total = (end - start).as_seconds_f64();
                if total <= 0.0 {
                    return 0.0;
                }
                ((time - start).as_seconds_f64() / total).clamp(0.0, 1.0)
            }
            _ => 0.0,
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    fn make_task(
        task_id: &str,
        try_number: u32,
        start: Option<OffsetDateTime>,
        end: Option<OffsetDateTime>,
        state: Option<TaskInstanceState>,
    ) -> super::super::TaskInstance {
        super::super::TaskInstance {
            task_id: task_id.into(),
            try_number,
            start_date: start,
            end_date: end,
            state,
            ..Default::default()
        }
    }

    #[test]
    fn test_empty_tasks() {
        let gantt = GanttData::from_task_instances(&[]);
        assert!(gantt.window_start.is_none());
        assert!(gantt.window_end.is_none());
        assert!(gantt.task_tries.is_empty());
    }

    #[test]
    fn test_single_task_window() {
        let tasks = vec![make_task(
            "task_1",
            1,
            Some(datetime!(2024-01-01 10:00:00 UTC)),
            Some(datetime!(2024-01-01 10:30:00 UTC)),
            Some(TaskInstanceState::Success),
        )];
        let gantt = GanttData::from_task_instances(&tasks);
        assert_eq!(gantt.window_start, Some(datetime!(2024-01-01 10:00:00 UTC)));
        assert_eq!(gantt.window_end, Some(datetime!(2024-01-01 10:30:00 UTC)));
    }

    #[test]
    fn test_running_task_uses_now_for_window_end() {
        let tasks = vec![make_task(
            "task_1",
            1,
            Some(datetime!(2024-01-01 10:00:00 UTC)),
            None,
            Some(TaskInstanceState::Running),
        )];
        let gantt = GanttData::from_task_instances(&tasks);
        assert!(gantt.window_end.is_some());
        // Window end should be approximately now
        assert!(gantt.window_end.unwrap() > datetime!(2024-01-01 10:00:00 UTC));
    }

    #[test]
    fn test_multiple_tasks_window_spans() {
        let tasks = vec![
            make_task(
                "task_1",
                1,
                Some(datetime!(2024-01-01 10:00:00 UTC)),
                Some(datetime!(2024-01-01 10:30:00 UTC)),
                Some(TaskInstanceState::Success),
            ),
            make_task(
                "task_2",
                1,
                Some(datetime!(2024-01-01 10:30:00 UTC)),
                Some(datetime!(2024-01-01 11:00:00 UTC)),
                Some(TaskInstanceState::Failed),
            ),
        ];
        let gantt = GanttData::from_task_instances(&tasks);
        assert_eq!(gantt.window_start, Some(datetime!(2024-01-01 10:00:00 UTC)));
        assert_eq!(gantt.window_end, Some(datetime!(2024-01-01 11:00:00 UTC)));
    }

    #[test]
    fn test_update_tries() {
        let tasks = vec![make_task(
            "task_1",
            2,
            Some(datetime!(2024-01-01 10:30:00 UTC)),
            Some(datetime!(2024-01-01 11:00:00 UTC)),
            Some(TaskInstanceState::Success),
        )];
        let mut gantt = GanttData::from_task_instances(&tasks);

        // Now add historical try data
        gantt.update_tries(
            &"task_1".into(),
            vec![
                TaskTryGantt {
                    try_number: 1,
                    start_date: Some(datetime!(2024-01-01 10:00:00 UTC)),
                    end_date: Some(datetime!(2024-01-01 10:20:00 UTC)),
                    state: Some(TaskInstanceState::Failed),
                },
                TaskTryGantt {
                    try_number: 2,
                    start_date: Some(datetime!(2024-01-01 10:30:00 UTC)),
                    end_date: Some(datetime!(2024-01-01 11:00:00 UTC)),
                    state: Some(TaskInstanceState::Success),
                },
            ],
        );

        let task_id: TaskId = "task_1".into();
        assert_eq!(gantt.task_tries.get(&task_id).unwrap().len(), 2);
        // Window should now span from first try start to second try end
        assert_eq!(gantt.window_start, Some(datetime!(2024-01-01 10:00:00 UTC)));
        assert_eq!(gantt.window_end, Some(datetime!(2024-01-01 11:00:00 UTC)));
    }
}
