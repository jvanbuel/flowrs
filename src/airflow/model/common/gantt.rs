use std::collections::HashMap;

use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use time::OffsetDateTime;

use crate::airflow::client::v1;
use crate::airflow::client::v2;

use super::taskinstance::TaskInstanceState;
use super::TaskId;
use crate::ui::constants::AirflowStateColor;

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
    /// whose `char_colors` overwrite each other in [`Self::create_bar`]. This is
    /// intentional for now; callers who need per-map-index bars should pre-group
    /// or provide distinct `TaskId` values.
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
    fn recompute_window(&mut self) {
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
    fn ratio(&self, time: OffsetDateTime) -> f64 {
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

    /// Create a Gantt bar `Line` for a specific task, sized to `width` characters.
    /// Each try renders as a colored segment; gaps between tries are empty.
    pub fn create_bar(&self, task_id: &TaskId, width: usize) -> Line<'static> {
        const FILLED_CHAR: &str = "▃";
        const EMPTY_CHAR: &str = " ";

        if width == 0 {
            return Line::default();
        }

        let Some(tries) = self.task_tries.get(task_id) else {
            return Line::from(EMPTY_CHAR.repeat(width));
        };

        if self.window_start.is_none() || self.window_end.is_none() {
            return Line::from(EMPTY_CHAR.repeat(width));
        }

        // Build a per-character color map
        // Each character position maps to either None (empty) or Some(Color)
        let mut char_colors: Vec<Option<Color>> = vec![None; width];

        for t in tries {
            let Some(start) = t.start_date else {
                continue;
            };
            let end = t.end_date.unwrap_or_else(OffsetDateTime::now_utc);

            let start_ratio = self.ratio(start);
            let end_ratio = self.ratio(end);

            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_precision_loss
            )]
            let start_col = (start_ratio * width as f64).floor() as usize;
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_precision_loss
            )]
            let end_col = (end_ratio * width as f64)
                .ceil()
                .max(start_col as f64 + 1.0) as usize;
            let end_col = end_col.min(width);

            let color: Color = t
                .state
                .as_ref()
                .map_or(AirflowStateColor::None, AirflowStateColor::from)
                .into();

            for slot in &mut char_colors[start_col..end_col] {
                *slot = Some(color);
            }
        }

        // Convert char_colors into spans by grouping consecutive same-color characters
        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut i = 0;
        while i < char_colors.len() {
            let current_color = char_colors[i];
            let mut count = 1;
            while i + count < char_colors.len() && char_colors[i + count] == current_color {
                count += 1;
            }
            match current_color {
                Some(color) => {
                    spans.push(Span::styled(
                        FILLED_CHAR.repeat(count),
                        Style::default().fg(color),
                    ));
                }
                None => {
                    spans.push(Span::raw(EMPTY_CHAR.repeat(count)));
                }
            }
            i += count;
        }

        Line::from(spans)
    }
}

// From trait implementations for v1 try response models
impl From<v1::model::taskinstance::TaskInstanceTryResponse> for TaskTryGantt {
    fn from(value: v1::model::taskinstance::TaskInstanceTryResponse) -> Self {
        Self {
            try_number: value.try_number,
            start_date: value.start_date,
            end_date: value.end_date,
            state: value.state.map(|s| TaskInstanceState::from(s.as_str())),
        }
    }
}

// From trait implementations for v2 try response models
impl From<v2::model::taskinstance::TaskInstanceTryResponse> for TaskTryGantt {
    fn from(value: v2::model::taskinstance::TaskInstanceTryResponse) -> Self {
        Self {
            try_number: value.try_number,
            start_date: value.start_date,
            end_date: value.end_date,
            state: value.state.map(|s| TaskInstanceState::from(s.as_str())),
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
    fn test_bar_creation_single_task() {
        let tasks = vec![make_task(
            "task_1",
            1,
            Some(datetime!(2024-01-01 10:00:00 UTC)),
            Some(datetime!(2024-01-01 11:00:00 UTC)),
            Some(TaskInstanceState::Success),
        )];
        let gantt = GanttData::from_task_instances(&tasks);
        let bar = gantt.create_bar(&"task_1".into(), 20);
        // Single task spanning entire window should fill all 20 chars
        let total_chars: usize = bar.spans.iter().map(|s| s.content.chars().count()).sum();
        assert_eq!(total_chars, 20);
    }

    #[test]
    fn test_bar_creation_missing_task() {
        let tasks = vec![make_task(
            "task_1",
            1,
            Some(datetime!(2024-01-01 10:00:00 UTC)),
            Some(datetime!(2024-01-01 11:00:00 UTC)),
            Some(TaskInstanceState::Success),
        )];
        let gantt = GanttData::from_task_instances(&tasks);
        let bar = gantt.create_bar(&"nonexistent".into(), 20);
        let total_chars: usize = bar.spans.iter().map(|s| s.content.chars().count()).sum();
        assert_eq!(total_chars, 20);
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

        // task_1 should fill the first half
        let bar = gantt.create_bar(&"task_1".into(), 20);
        let total_chars: usize = bar.spans.iter().map(|s| s.content.chars().count()).sum();
        assert_eq!(total_chars, 20);
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
