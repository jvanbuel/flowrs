use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use time::OffsetDateTime;

use crate::airflow::model::common::gantt::GanttData;
use crate::airflow::model::common::TaskId;

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

    for t in tries {
        if !t.is_renderable() {
            continue;
        }
        // Safe: is_renderable() guarantees start_date is Some.
        let start = t.start_date.expect("renderable try has start_date");
        let end = t.end_date.unwrap_or_else(OffsetDateTime::now_utc);

        let start_ratio = gantt.ratio(start);
        let end_ratio = gantt.ratio(end);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airflow::model::common::gantt::GanttData;
    use crate::airflow::model::common::taskinstance::TaskInstanceState;
    use crate::airflow::model::common::TaskInstance;
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
    fn test_bar_skips_pending_try_after_clear() {
        // Simulates a cleared task: the Gantt has two tries for task_1 —
        // try 1 succeeded normally, try 2 is the new pending instance with a
        // start_date populated but no end_date and a non-running state.
        // The bar must color only try 1's segment; try 2 must be skipped.
        use crate::airflow::model::common::gantt::TaskTryGantt;

        let mut gantt = GanttData::from_task_instances(&[make_task(
            "task_1",
            1,
            Some(datetime!(2024-01-01 10:00:00 UTC)),
            Some(datetime!(2024-01-01 10:30:00 UTC)),
            Some(TaskInstanceState::Success),
        )]);
        gantt.task_tries.insert(
            "task_1".into(),
            vec![
                TaskTryGantt {
                    try_number: 1,
                    start_date: Some(datetime!(2024-01-01 10:00:00 UTC)),
                    end_date: Some(datetime!(2024-01-01 10:30:00 UTC)),
                    state: Some(TaskInstanceState::Success),
                },
                TaskTryGantt {
                    try_number: 2,
                    start_date: Some(datetime!(2024-01-01 10:30:00 UTC)),
                    end_date: None,
                    state: Some(TaskInstanceState::Scheduled),
                },
            ],
        );
        gantt.recompute_window();

        let bar = create_gantt_bar(&gantt, &"task_1".into(), 20);
        let total_chars: usize = bar.spans.iter().map(|s| s.content.chars().count()).sum();
        assert_eq!(total_chars, 20);
        // Every colored char in the bar must come from the Success try, not
        // the pending one — i.e. no span should be styled with a color other
        // than the Success color (raw spans = empty).
        let success_color: ratatui::style::Color = AirflowStateColor::Success.into();
        for span in &bar.spans {
            if let Some(fg) = span.style.fg {
                assert_eq!(
                    fg, success_color,
                    "unexpected color in bar after cleared try: {span:?}"
                );
            }
        }
    }

    #[test]
    fn test_bar_keeps_success_color_when_cleared_try_has_stale_dates() {
        // Reproduces the user-reported bug: previously successful try gets
        // repainted gray after clearing. The /tries endpoint returns both
        // the historical success AND the cleared TI with stale dates but
        // state=None. The bar must keep the success color, not gray.
        use crate::airflow::model::common::gantt::TaskTryGantt;

        let mut gantt = GanttData::default();
        gantt.task_tries.insert(
            "task_1".into(),
            vec![
                TaskTryGantt {
                    try_number: 1,
                    start_date: Some(datetime!(2024-01-01 10:00:00 UTC)),
                    end_date: Some(datetime!(2024-01-01 10:30:00 UTC)),
                    state: Some(TaskInstanceState::Success),
                },
                // Cleared TI: state reset, but Airflow kept stale dates.
                TaskTryGantt {
                    try_number: 2,
                    start_date: Some(datetime!(2024-01-01 10:00:00 UTC)),
                    end_date: Some(datetime!(2024-01-01 10:30:00 UTC)),
                    state: None,
                },
            ],
        );
        gantt.recompute_window();

        let bar = create_gantt_bar(&gantt, &"task_1".into(), 20);
        let success_color: ratatui::style::Color = AirflowStateColor::Success.into();
        let none_color: ratatui::style::Color = AirflowStateColor::None.into();
        for span in &bar.spans {
            if let Some(fg) = span.style.fg {
                assert_ne!(
                    fg, none_color,
                    "bar painted with None color after clear: {span:?}"
                );
                assert_eq!(fg, success_color, "unexpected color in bar: {span:?}");
            }
        }
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
