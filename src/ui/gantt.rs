use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use time::OffsetDateTime;

use crate::airflow::model::common::gantt::GanttData;
use crate::airflow::model::common::taskinstance::TaskInstanceState;
use crate::airflow::model::common::TaskId;

use super::common::state_to_colored_square;
use super::constants::AirflowStateColor;

/// Create a Gantt bar `Line` for a specific task, sized to `width` characters.
/// Each try renders as a colored segment; gaps between tries are empty.
pub fn create_gantt_bar(gantt: &GanttData, task_id: &TaskId, width: usize) -> Line<'static> {
    const FILLED_CHAR: &str = "▃";
    const EMPTY_CHAR: &str = " ";

    if width == 0 {
        return Line::default();
    }

    let Some(tries) = gantt.task_tries.get(task_id) else {
        return Line::from(EMPTY_CHAR.repeat(width));
    };

    if gantt.window_start.is_none() || gantt.window_end.is_none() {
        return Line::from(EMPTY_CHAR.repeat(width));
    }

    // Build a per-character color map
    // Each character position maps to either None (empty) or Some(Color)
    let mut char_colors: Vec<Option<Color>> = vec![None; width];

    // Paint a `[seg_start, seg_end)` time span into the color map with `color`.
    let paint = |char_colors: &mut [Option<Color>], seg_start: OffsetDateTime, seg_end, color| {
        if seg_end <= seg_start {
            return;
        }
        let start_ratio = gantt.ratio(seg_start);
        let end_ratio = gantt.ratio(seg_end);

        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let start_col = ((start_ratio * width as f64).floor() as usize).min(width);
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let end_col = ((end_ratio * width as f64)
            .ceil()
            .max(start_col as f64 + 1.0) as usize)
            .min(width);

        for slot in &mut char_colors[start_col..end_col] {
            *slot = Some(color);
        }
    };

    let now = OffsetDateTime::now_utc();
    let queued_color: Color = AirflowStateColor::Queued.into();
    let scheduled_color: Color = AirflowStateColor::Scheduled.into();

    for t in tries {
        // Lead-in phases are painted before the running segment so that, on any
        // overlap, the running (state-colored) bar wins.

        // Scheduled: from `scheduled_when` until the task is queued or starts.
        if let Some(scheduled) = t.scheduled_when {
            let end = t.queued_when.or(t.start_date).unwrap_or(now);
            paint(&mut char_colors, scheduled, end, scheduled_color);
        }

        // Queued: from `queued_when` until the task starts running.
        if let Some(queued) = t.queued_when {
            let end = t.start_date.unwrap_or(now);
            paint(&mut char_colors, queued, end, queued_color);
        }

        // Running: from `start_date` until the task ends (or now, if in progress).
        if let Some(start) = t.start_date {
            let end = t.end_date.unwrap_or(now);
            let color: Color = t
                .state
                .as_ref()
                .map_or(AirflowStateColor::None, AirflowStateColor::from)
                .into();
            paint(&mut char_colors, start, end, color);
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

/// Single-line Gantt color key — the two lead-in phases followed by every task
/// state the run segment can take — rendered as a legend beneath the
/// task-instance Gantt column.
///
/// Uses the same `■` swatch as the table's State column (`state_to_colored_square`)
/// so the two line up visually. Labels come from each state's `Display` and
/// colors from the `TaskInstanceState -> AirflowStateColor` mapping, so the
/// legend stays in sync with how the bar itself is colored. States that map to
/// no distinct color (e.g. `Deferred`, `Removed`) are intentionally omitted.
///
/// Each swatch is joined to its label with a non-breaking space (`\u{a0}`) while
/// items are separated by regular spaces, so a `Paragraph` with `Wrap` reflows
/// the legend onto multiple rows on narrow panels without ever splitting a
/// swatch from its label.
pub fn gantt_legend_line() -> Line<'static> {
    // In the order Airflow phases progress, then terminal states.
    let states = [
        TaskInstanceState::Scheduled,
        TaskInstanceState::Queued,
        TaskInstanceState::Running,
        TaskInstanceState::Success,
        TaskInstanceState::Failed,
        TaskInstanceState::UpForRetry,
        TaskInstanceState::UpForReschedule,
        TaskInstanceState::Skipped,
        TaskInstanceState::UpstreamFailed,
    ];
    let mut spans: Vec<Span<'static>> = Vec::with_capacity(states.len() * 2);
    for state in &states {
        spans.push(state_to_colored_square(AirflowStateColor::from(state)));
        spans.push(Span::raw(format!("\u{a0}{state}  ")));
    }
    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airflow::model::common::gantt::GanttData;
    use crate::airflow::model::common::taskinstance::TaskInstanceState;
    use crate::airflow::model::common::TaskInstance;
    use crate::ui::constants::AirflowStateColor;
    use time::macros::datetime;

    fn make_task(
        task_id: &str,
        try_number: u32,
        start: Option<OffsetDateTime>,
        end: Option<OffsetDateTime>,
        state: Option<TaskInstanceState>,
    ) -> TaskInstance {
        TaskInstance {
            task_id: task_id.into(),
            try_number,
            start_date: start,
            end_date: end,
            state,
            ..Default::default()
        }
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
        let bar = create_gantt_bar(&gantt, &"task_1".into(), 20);
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
        let bar = create_gantt_bar(&gantt, &"nonexistent".into(), 20);
        let total_chars: usize = bar.spans.iter().map(|s| s.content.chars().count()).sum();
        assert_eq!(total_chars, 20);
    }

    #[test]
    fn test_bar_renders_queued_and_scheduled_segments() {
        // A task scheduled at 10:00, queued at 10:15, started at 10:30, ended at 11:00.
        // The window spans 10:00..11:00, so each phase occupies a quarter of the bar.
        let task = TaskInstance {
            task_id: "task_1".into(),
            try_number: 1,
            scheduled_when: Some(datetime!(2024-01-01 10:00:00 UTC)),
            queued_when: Some(datetime!(2024-01-01 10:15:00 UTC)),
            start_date: Some(datetime!(2024-01-01 10:30:00 UTC)),
            end_date: Some(datetime!(2024-01-01 11:00:00 UTC)),
            state: Some(TaskInstanceState::Success),
            ..Default::default()
        };
        let gantt = GanttData::from_task_instances(&[task]);
        // Window starts at the scheduled time, not the start time.
        assert_eq!(gantt.window_start, Some(datetime!(2024-01-01 10:00:00 UTC)));

        let bar = create_gantt_bar(&gantt, &"task_1".into(), 20);
        let total_chars: usize = bar.spans.iter().map(|s| s.content.chars().count()).sum();
        assert_eq!(total_chars, 20);

        // Expect three distinct colors: scheduled (tan), queued (gray), running (state).
        let scheduled: Color = AirflowStateColor::Scheduled.into();
        let queued: Color = AirflowStateColor::Queued.into();
        let running: Color = AirflowStateColor::Success.into();
        let colors: Vec<Option<Color>> = bar
            .spans
            .iter()
            .flat_map(|s| std::iter::repeat_n(s.style.fg, s.content.chars().count()))
            .collect();
        assert!(
            colors.contains(&Some(scheduled)),
            "scheduled segment missing"
        );
        assert!(colors.contains(&Some(queued)), "queued segment missing");
        assert!(colors.contains(&Some(running)), "running segment missing");
        // Ordering: scheduled comes before queued comes before running.
        let first = |c: Color| colors.iter().position(|&x| x == Some(c)).unwrap();
        assert!(first(scheduled) < first(queued));
        assert!(first(queued) < first(running));
    }

    #[test]
    fn test_airflow_v2_bar_has_no_gap_between_queued_and_running() {
        // Airflow 2 (v2 API) has no `scheduled_when`: a task only carries a
        // queued time followed by the run. The bar must show queued directly
        // adjacent to running, with no blank cells in between.
        let task = TaskInstance {
            task_id: "task_1".into(),
            try_number: 1,
            scheduled_when: None,
            queued_when: Some(datetime!(2024-01-01 10:00:00 UTC)),
            start_date: Some(datetime!(2024-01-01 10:20:00 UTC)),
            end_date: Some(datetime!(2024-01-01 11:00:00 UTC)),
            state: Some(TaskInstanceState::Success),
            ..Default::default()
        };
        let gantt = GanttData::from_task_instances(&[task]);
        // The window starts at the queued time, so the bar fills from the left.
        assert_eq!(gantt.window_start, Some(datetime!(2024-01-01 10:00:00 UTC)));

        let bar = create_gantt_bar(&gantt, &"task_1".into(), 20);
        let colors: Vec<Option<Color>> = bar
            .spans
            .iter()
            .flat_map(|s| std::iter::repeat_n(s.style.fg, s.content.chars().count()))
            .collect();

        // No empty cell appears between the first and last painted cell.
        let first_filled = colors.iter().position(Option::is_some).unwrap();
        let last_filled = colors.iter().rposition(Option::is_some).unwrap();
        assert!(
            colors[first_filled..=last_filled]
                .iter()
                .all(Option::is_some),
            "found a gap between queued and running segments: {colors:?}"
        );
        // And both phases are present (queued then running), no scheduled tan.
        let queued: Color = AirflowStateColor::Queued.into();
        let running: Color = AirflowStateColor::Success.into();
        let scheduled: Color = AirflowStateColor::Scheduled.into();
        assert!(colors.contains(&Some(queued)), "queued segment missing");
        assert!(colors.contains(&Some(running)), "running segment missing");
        assert!(
            !colors.contains(&Some(scheduled)),
            "v2 task should not paint a scheduled segment"
        );
    }

    #[test]
    fn test_bar_renders_queued_segment_for_not_yet_started_task() {
        // A task currently queued (no start_date) should still show a bar.
        let task = TaskInstance {
            task_id: "task_1".into(),
            try_number: 1,
            queued_when: Some(datetime!(2024-01-01 10:00:00 UTC)),
            start_date: None,
            end_date: None,
            state: Some(TaskInstanceState::Queued),
            ..Default::default()
        };
        let gantt = GanttData::from_task_instances(&[task]);
        let bar = create_gantt_bar(&gantt, &"task_1".into(), 20);
        let queued: Color = AirflowStateColor::Queued.into();
        let has_queued = bar.spans.iter().any(|s| s.style.fg == Some(queued));
        assert!(has_queued, "queued task should render a queued segment");
    }

    #[test]
    fn test_bar_creation_multiple_tasks() {
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

        // task_1 should fill the first half
        let bar = create_gantt_bar(&gantt, &"task_1".into(), 20);
        let total_chars: usize = bar.spans.iter().map(|s| s.content.chars().count()).sum();
        assert_eq!(total_chars, 20);
    }
}
