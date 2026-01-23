use crossterm::event::KeyCode;
use log::debug;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation,
    ScrollbarState, StatefulWidget, Table, Widget, Wrap,
};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use time::format_description;

use crate::airflow::model::common::{calculate_duration, format_duration, DagRun};
use crate::app::events::custom::FlowrsEvent;
use crate::ui::common::create_headers;
use crate::ui::constants::AirflowStateColor;
use crate::ui::theme::{
    BORDER_STYLE, DEFAULT_STYLE, SELECTED_ROW_STYLE, SURFACE_STYLE, TABLE_HEADER_STYLE, TITLE_STYLE,
};
use crate::ui::TIME_FORMAT;

use super::popup::dagruns::commands::DAGRUN_COMMAND_POP_UP;
use super::popup::dagruns::trigger::TriggerDagRunPopUp;
use super::popup::dagruns::DagRunPopUp;
use super::popup::popup_area;
use super::popup::{dagruns::clear::ClearDagRunPopup, dagruns::mark::MarkDagRunPopup};
use super::{FilterableTable, KeyResult, Model, Popup};
use crate::app::worker::{OpenItem, WorkerMessage};

/// Model for the DAG Run panel, managing the list of DAG runs and their filtering.
pub struct DagRunModel {
    pub dag_id: Option<String>,
    pub dag_code: DagCodeWidget,
    /// Filterable table containing all DAG runs and filtered view
    pub table: FilterableTable<DagRun>,
    /// Unified popup state (error, commands, or custom for this model)
    pub popup: Popup<DagRunPopUp>,
    ticks: u32,
    event_buffer: Vec<KeyCode>,
}

impl Default for DagRunModel {
    fn default() -> Self {
        Self {
            dag_id: None,
            dag_code: DagCodeWidget::default(),
            table: FilterableTable::new(),
            popup: Popup::None,
            ticks: 0,
            event_buffer: Vec::new(),
        }
    }
}

#[derive(Default)]
pub struct DagCodeWidget {
    pub cached_lines: Option<Vec<Line<'static>>>,
    pub vertical_scroll: usize,
    pub vertical_scroll_state: ScrollbarState,
}

impl DagCodeWidget {
    pub fn set_code(&mut self, code: &str) {
        self.cached_lines = Some(code_to_lines(code));
        self.vertical_scroll = 0;
        self.vertical_scroll_state = ScrollbarState::default();
    }

    pub fn clear(&mut self) {
        self.cached_lines = None;
        self.vertical_scroll = 0;
        self.vertical_scroll_state = ScrollbarState::default();
    }
}

impl DagRunModel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a text-based duration gauge line.
    /// The gauge normalizes durations to show relative progress within visible items.
    fn create_duration_gauge(
        duration_seconds: f64,
        max_duration: f64,
        color: ratatui::style::Color,
        width: usize,
    ) -> Line<'static> {
        const FILLED_CHAR: &str = "─";
        const EMPTY_CHAR: &str = " ";

        // Calculate the ratio (0.0 to 1.0)
        let ratio = if max_duration > 0.0 {
            (duration_seconds / max_duration).min(1.0)
        } else {
            0.0
        };

        // Calculate how many characters should be filled
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let filled_width = (ratio * width as f64).round() as usize;
        let empty_width = width.saturating_sub(filled_width);

        // Create the gauge string
        let filled = FILLED_CHAR.repeat(filled_width);
        let empty = EMPTY_CHAR.repeat(empty_width);

        Line::from(vec![
            Span::styled(filled, Style::default().fg(color).bold()),
            Span::styled(empty, Style::default().fg(color).dim()),
        ])
    }

    /// Sort filtered DAG runs by `logical_date` descending
    /// Call this after `apply_filter()` to ensure proper ordering
    pub fn sort_dag_runs(&mut self) {
        // Sort by logical_date (execution date) descending, with fallback to start_date
        // This ensures queued runs (which have no start_date yet) appear in chronological order
        self.table.filtered.items.sort_by(|a, b| {
            b.logical_date
                .or(b.start_date)
                .cmp(&a.logical_date.or(a.start_date))
        });
    }

    /// Get the currently selected DAG run
    pub fn current(&self) -> Option<&DagRun> {
        self.table.current()
    }

    /// Returns selected DAG run IDs for passing to mark/clear popups
    fn selected_dag_run_ids(&self) -> Vec<String> {
        self.table.selected_ids(|item| item.dag_run_id.clone())
    }

    /// Mark a DAG run with a new status (optimistic update)
    pub fn mark_dag_run(&mut self, dag_run_id: &str, status: &str) {
        if let Some(dag_run) = self
            .table
            .filtered
            .items
            .iter_mut()
            .find(|dr| dr.dag_run_id == dag_run_id)
        {
            dag_run.state = status.to_string();
        }
    }
}

impl DagRunModel {
    /// Handle dag code viewer navigation
    fn handle_dag_code_viewer(&mut self, key_code: KeyCode) -> KeyResult {
        if self.dag_code.cached_lines.is_none() {
            return KeyResult::Ignored;
        }
        match key_code {
            KeyCode::Esc | KeyCode::Char('q' | 'v') | KeyCode::Enter => {
                self.dag_code.clear();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.dag_code.vertical_scroll = self.dag_code.vertical_scroll.saturating_add(1);
                self.dag_code.vertical_scroll_state = self
                    .dag_code
                    .vertical_scroll_state
                    .position(self.dag_code.vertical_scroll);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.dag_code.vertical_scroll = self.dag_code.vertical_scroll.saturating_sub(1);
                self.dag_code.vertical_scroll_state = self
                    .dag_code
                    .vertical_scroll_state
                    .position(self.dag_code.vertical_scroll);
            }
            _ => {}
        }
        KeyResult::Consumed
    }

    /// Handle model-specific popups (returns messages from popup)
    fn handle_popup(&mut self, event: &FlowrsEvent) -> Option<Vec<WorkerMessage>> {
        let custom_popup = self.popup.custom_mut()?;
        let (key_event, messages) = match custom_popup {
            DagRunPopUp::Clear(p) => p.update(event),
            DagRunPopUp::Mark(p) => p.update(event),
            DagRunPopUp::Trigger(p) => p.update(event),
        };
        debug!("Popup messages: {messages:?}");

        if let Some(FlowrsEvent::Key(key_event)) = &key_event {
            if matches!(
                key_event.code,
                KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q')
            ) {
                let exit_visual =
                    matches!(custom_popup, DagRunPopUp::Clear(_) | DagRunPopUp::Mark(_));
                self.popup.close();
                if exit_visual {
                    self.table.visual_mode = false;
                    self.table.visual_anchor = None;
                }
            }
        }
        Some(messages)
    }

    /// Handle model-specific keys
    fn handle_keys(&mut self, key_code: KeyCode) -> KeyResult {
        match key_code {
            KeyCode::Char('t') => {
                if let Some(dag_id) = &self.dag_id {
                    self.popup
                        .show_custom(DagRunPopUp::Trigger(TriggerDagRunPopUp::new(
                            dag_id.clone(),
                        )));
                }
                KeyResult::Consumed
            }
            KeyCode::Char('m') => {
                let dag_run_ids = self.selected_dag_run_ids();
                if let Some(dag_id) = &self.dag_id {
                    if !dag_run_ids.is_empty() {
                        self.popup
                            .show_custom(DagRunPopUp::Mark(MarkDagRunPopup::new(
                                dag_run_ids,
                                dag_id.clone(),
                            )));
                    }
                }
                KeyResult::Consumed
            }
            KeyCode::Char('?') => {
                self.popup.show_commands(&DAGRUN_COMMAND_POP_UP);
                KeyResult::Consumed
            }
            KeyCode::Char('v') => {
                if let Some(dag_id) = &self.dag_id {
                    KeyResult::ConsumedWith(vec![WorkerMessage::GetDagCode {
                        dag_id: dag_id.clone(),
                    }])
                } else {
                    KeyResult::Consumed
                }
            }
            KeyCode::Char('c') => {
                let dag_run_ids = self.selected_dag_run_ids();
                if let Some(dag_id) = &self.dag_id {
                    if !dag_run_ids.is_empty() {
                        self.popup
                            .show_custom(DagRunPopUp::Clear(ClearDagRunPopup::new(
                                dag_run_ids,
                                dag_id.clone(),
                            )));
                    }
                }
                KeyResult::Consumed
            }
            KeyCode::Enter => {
                if let (Some(dag_id), Some(dag_run)) = (&self.dag_id, &self.current()) {
                    KeyResult::PassWith(vec![
                        WorkerMessage::UpdateTasks {
                            dag_id: dag_id.clone(),
                        },
                        WorkerMessage::UpdateTaskInstances {
                            dag_id: dag_id.clone(),
                            dag_run_id: dag_run.dag_run_id.clone(),
                            clear: true,
                        },
                    ])
                } else {
                    KeyResult::Consumed
                }
            }
            KeyCode::Char('o') => {
                if let (Some(dag_id), Some(dag_run)) = (&self.dag_id, &self.current()) {
                    KeyResult::PassWith(vec![WorkerMessage::OpenItem(OpenItem::DagRun {
                        dag_id: dag_id.clone(),
                        dag_run_id: dag_run.dag_run_id.clone(),
                    })])
                } else {
                    KeyResult::Consumed
                }
            }
            _ => KeyResult::PassThrough,
        }
    }
}

impl Model for DagRunModel {
    fn update(&mut self, event: &FlowrsEvent) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        match event {
            FlowrsEvent::Tick => {
                self.ticks += 1;
                if !self.ticks.is_multiple_of(10) {
                    return (Some(FlowrsEvent::Tick), vec![]);
                }
                let worker_messages = if let Some(dag_id) = &self.dag_id {
                    vec![WorkerMessage::UpdateDagRuns {
                        dag_id: dag_id.clone(),
                        clear: false,
                    }]
                } else {
                    Vec::default()
                };
                (Some(FlowrsEvent::Tick), worker_messages)
            }
            FlowrsEvent::Key(key_event) => {
                // Filter needs special handling - apply filter then sort
                if matches!(
                    self.table.handle_filter_key(key_event),
                    KeyResult::Consumed | KeyResult::ConsumedWith(_)
                ) {
                    self.sort_dag_runs();
                    return (None, vec![]);
                }

                // Popup handling (has its own update method)
                if let Some(messages) = self.handle_popup(event) {
                    return (None, messages);
                }

                // Chain the remaining handlers
                let result = self
                    .popup
                    .handle_dismiss(key_event.code)
                    .or_else(|| self.handle_dag_code_viewer(key_event.code))
                    .or_else(|| self.table.handle_visual_mode_key(key_event.code))
                    .or_else(|| {
                        self.table
                            .handle_navigation(key_event.code, &mut self.event_buffer)
                    })
                    .or_else(|| self.handle_keys(key_event.code));

                result.into_result(event)
            }
            FlowrsEvent::Mouse | FlowrsEvent::FocusGained | FlowrsEvent::FocusLost => {
                (Some(event.clone()), vec![])
            }
        }
    }
}

impl Widget for &mut DagRunModel {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let content_area = self.table.render_with_filter(area, buf);

        let headers = [
            "State",
            "DAG Run ID",
            "Logical Date",
            "Type",
            "Duration",
            "Time",
        ];
        let header_row = create_headers(headers);
        let header = Row::new(header_row).style(TABLE_HEADER_STYLE);

        // Calculate max duration for normalization
        let max_duration = self
            .table
            .filtered
            .items
            .iter()
            .filter_map(calculate_duration)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(1.0);

        // Calculate the width available for the Duration column
        // Total width - borders(2) - state(6) - dag_run_id(variable) - logical_date(20) - type(11) - time(10)
        let table_inner_width = content_area.width.saturating_sub(2); // Subtract borders
        let fixed_columns_width = 6 + 20 + 11 + 10 + 10; // State + Logical Date + Type + Time + spacing
        let dag_run_id_width = 30; // Fixed width for dag_run_id
        let gauge_width = table_inner_width
            .saturating_sub(fixed_columns_width + dag_run_id_width)
            .max(10) as usize;

        let rows = self
            .table
            .filtered
            .items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                let state_color = match item.state.as_str() {
                    "success" => AirflowStateColor::Success.into(),
                    "running" => AirflowStateColor::Running.into(),
                    "failed" => AirflowStateColor::Failed.into(),
                    "queued" => AirflowStateColor::Queued.into(),
                    _ => AirflowStateColor::None.into(),
                };

                let (duration_cell, time_cell) = if let Some(duration) = calculate_duration(item) {
                    (
                        DagRunModel::create_duration_gauge(
                            duration,
                            max_duration,
                            state_color,
                            gauge_width,
                        ),
                        Line::from(format_duration(duration)),
                    )
                } else {
                    (Line::from("-"), Line::from("-"))
                };

                Row::new(vec![
                    Line::from(Span::styled("■", Style::default().fg(state_color))),
                    Line::from(Span::styled(
                        item.dag_run_id.as_str(),
                        Style::default().add_modifier(Modifier::BOLD),
                    )),
                    Line::from(if let Some(date) = item.logical_date {
                        date.format(
                            &format_description::parse(TIME_FORMAT)
                                .expect("TIME_FORMAT constant should be a valid time format"),
                        )
                        .expect("Date formatting with TIME_FORMAT should succeed")
                    } else {
                        "None".to_string()
                    }),
                    Line::from(item.run_type.as_str()),
                    duration_cell,
                    time_cell,
                ])
                .style(self.table.row_style(idx))
            });
        let t = Table::new(
            rows,
            &[
                Constraint::Length(6),  // State
                Constraint::Length(30), // DAG Run ID
                Constraint::Length(20), // Logical Date
                Constraint::Length(11), // Type
                Constraint::Fill(1),    // Duration gauge (expands)
                Constraint::Length(10), // Time (formatted duration)
            ],
        )
        .header(header)
        .block({
            let block = Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                .border_style(BORDER_STYLE)
                .title(" Press <?> to see available commands ");
            if let Some(title) = self.table.status_title() {
                block.title_bottom(title)
            } else {
                block
            }
        })
        .row_highlight_style(SELECTED_ROW_STYLE);
        StatefulWidget::render(t, content_area, buf, &mut self.table.filtered.state);

        if let Some(cached_lines) = &self.dag_code.cached_lines {
            let area = popup_area(area, 60, 90);

            let popup = Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .title(" DAG Code ")
                .border_style(BORDER_STYLE)
                .style(SURFACE_STYLE)
                .title_style(TITLE_STYLE);

            #[allow(clippy::cast_possible_truncation)]
            let code_text = Paragraph::new(cached_lines.clone())
                .block(popup)
                .style(DEFAULT_STYLE)
                .wrap(Wrap { trim: false })
                .scroll((self.dag_code.vertical_scroll as u16, 0));

            Clear.render(area, buf); //this clears out the background
            code_text.render(area, buf);

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            scrollbar.render(area, buf, &mut self.dag_code.vertical_scroll_state);
        }

        // Render any active popup (error, commands, or custom)
        (&self.popup).render(area, buf);

        // Render custom popups that need special handling
        match self.popup.custom_mut() {
            Some(DagRunPopUp::Clear(popup)) => popup.render(area, buf),
            Some(DagRunPopUp::Mark(popup)) => popup.render(area, buf),
            Some(DagRunPopUp::Trigger(popup)) => popup.render(area, buf),
            None => {}
        }
    }
}

fn code_to_lines(dag_code: &str) -> Vec<Line<'static>> {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let syntax = ps
        .find_syntax_by_extension("py")
        .expect("Python syntax definition should be available in default syntax set");
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    let mut lines: Vec<Line<'static>> = vec![];
    for line in LinesWithEndings::from(dag_code) {
        // LinesWithEndings enables use of newlines mode
        let line_spans: Vec<Span<'static>> = h
            .highlight_line(line, &ps)
            .expect("Syntax highlighting should succeed for valid Python code")
            .into_iter()
            .map(|(style, text)| {
                let fg = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                Span::styled(text.to_string(), Style::default().fg(fg))
            })
            .collect();
        lines.push(Line::from(line_spans));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::model::filter::Filterable;
    use crossterm::event::{KeyEvent, KeyModifiers};
    use time::macros::datetime;

    #[test]
    fn test_duration_gauge_ratios() {
        let gauge_full =
            DagRunModel::create_duration_gauge(100.0, 100.0, ratatui::style::Color::Green, 10);
        let gauge_half =
            DagRunModel::create_duration_gauge(50.0, 100.0, ratatui::style::Color::Green, 10);
        let gauge_empty =
            DagRunModel::create_duration_gauge(0.0, 100.0, ratatui::style::Color::Green, 10);

        assert_eq!(gauge_full.spans[0].content.chars().count(), 10);
        assert_eq!(gauge_half.spans[0].content.chars().count(), 5);
        assert_eq!(gauge_empty.spans[0].content.chars().count(), 0);
    }

    #[test]
    fn test_duration_gauge_edge_cases() {
        // Zero max should not panic
        let gauge = DagRunModel::create_duration_gauge(50.0, 0.0, ratatui::style::Color::Green, 10);
        assert_eq!(gauge.spans[0].content.chars().count(), 0);

        // Duration exceeding max should cap at 100%
        let gauge =
            DagRunModel::create_duration_gauge(150.0, 100.0, ratatui::style::Color::Green, 10);
        assert_eq!(gauge.spans[0].content.chars().count(), 10);
    }

    #[test]
    fn test_sort_dag_runs_by_logical_date() {
        // Create test DAG runs with different states and dates
        let mut model = DagRunModel::new();

        // Oldest run: completed (has both logical_date and start_date)
        let oldest_run = DagRun {
            dag_id: "test_dag".to_string(),
            dag_run_id: "run_1".to_string(),
            logical_date: Some(datetime!(2024-01-01 10:00:00 UTC)),
            start_date: Some(datetime!(2024-01-01 10:05:00 UTC)),
            end_date: Some(datetime!(2024-01-01 10:30:00 UTC)),
            state: "success".to_string(),
            run_type: "scheduled".to_string(),
            ..Default::default()
        };

        // Middle run: queued (has logical_date but no start_date)
        let queued_run = DagRun {
            dag_id: "test_dag".to_string(),
            dag_run_id: "run_2".to_string(),
            logical_date: Some(datetime!(2024-01-02 10:00:00 UTC)),
            start_date: None, // Queued runs don't have start_date
            end_date: None,
            state: "queued".to_string(),
            run_type: "scheduled".to_string(),
            ..Default::default()
        };

        // Newest run: running (has both logical_date and start_date)
        let newest_run = DagRun {
            dag_id: "test_dag".to_string(),
            dag_run_id: "run_3".to_string(),
            logical_date: Some(datetime!(2024-01-03 10:00:00 UTC)),
            start_date: Some(datetime!(2024-01-03 10:05:00 UTC)),
            end_date: None,
            state: "running".to_string(),
            run_type: "scheduled".to_string(),
            ..Default::default()
        };

        // Add runs in random order to test sorting
        model.table.all = vec![oldest_run, newest_run, queued_run];

        // Apply filter then sort
        model.table.apply_filter();
        model.sort_dag_runs();

        // Verify runs are sorted by logical_date descending (newest first)
        assert_eq!(model.table.filtered.items.len(), 3);
        assert_eq!(model.table.filtered.items[0].dag_run_id, "run_3"); // Newest
        assert_eq!(model.table.filtered.items[1].dag_run_id, "run_2"); // Queued (middle)
        assert_eq!(model.table.filtered.items[2].dag_run_id, "run_1"); // Oldest
    }

    #[test]
    fn test_sort_dag_runs_fallback_to_start_date() {
        let mut model = DagRunModel::new();

        // Run with only start_date (logical_date is None)
        let run_with_start = DagRun {
            dag_id: "test_dag".to_string(),
            dag_run_id: "run_1".to_string(),
            logical_date: None,
            start_date: Some(datetime!(2024-01-02 10:00:00 UTC)),
            state: "running".to_string(),
            run_type: "manual".to_string(),
            ..Default::default()
        };

        // Run with both dates
        let run_with_both = DagRun {
            dag_id: "test_dag".to_string(),
            dag_run_id: "run_2".to_string(),
            logical_date: Some(datetime!(2024-01-01 10:00:00 UTC)),
            start_date: Some(datetime!(2024-01-01 10:00:00 UTC)),
            state: "success".to_string(),
            run_type: "scheduled".to_string(),
            ..Default::default()
        };

        model.table.all = vec![run_with_both, run_with_start];
        model.table.apply_filter();
        model.sort_dag_runs();

        // run_with_start should be first (newer start_date used as fallback)
        assert_eq!(model.table.filtered.items.len(), 2);
        assert_eq!(model.table.filtered.items[0].dag_run_id, "run_1");
        assert_eq!(model.table.filtered.items[1].dag_run_id, "run_2");
    }

    #[test]
    fn test_filter_and_sort_dag_runs_with_prefix() {
        let mut model = DagRunModel::new();

        let run_manual = DagRun {
            dag_id: "test_dag".to_string(),
            dag_run_id: "manual_run_1".to_string(),
            logical_date: Some(datetime!(2024-01-02 10:00:00 UTC)),
            state: "success".to_string(),
            run_type: "manual".to_string(),
            ..Default::default()
        };

        let run_scheduled = DagRun {
            dag_id: "test_dag".to_string(),
            dag_run_id: "scheduled_run_1".to_string(),
            logical_date: Some(datetime!(2024-01-03 10:00:00 UTC)),
            state: "queued".to_string(),
            run_type: "scheduled".to_string(),
            ..Default::default()
        };

        model.table.all = vec![run_manual, run_scheduled];

        // Filter by typing "manual" (activate filter and type)
        model.table.filter.activate();
        for c in "manual".chars() {
            model.table.filter.update(
                &KeyEvent::new(crossterm::event::KeyCode::Char(c), KeyModifiers::empty()),
                &DagRun::filterable_fields(),
            );
        }
        model.table.apply_filter();
        model.sort_dag_runs();

        // Should only show the manual run
        assert_eq!(model.table.filtered.items.len(), 1);
        assert_eq!(model.table.filtered.items[0].dag_run_id, "manual_run_1");
    }
}
