use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use time::OffsetDateTime;

use flowrs_airflow_model::model::common::gantt::GanttData;
use flowrs_airflow_model::model::common::TaskId;

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
        let Some(start) = t.start_date else {
            continue;
        };
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
    use flowrs_airflow_model::model::common::gantt::GanttData;
    use flowrs_airflow_model::model::common::taskinstance::TaskInstanceState;
    use flowrs_airflow_model::model::common::TaskInstance;
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
