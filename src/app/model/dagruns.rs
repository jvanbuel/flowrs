use crossterm::event::KeyCode;
use log::debug;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation,
    ScrollbarState, StatefulWidget, Table, Widget, Wrap,
};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use syntect_tui::into_span;
use time::format_description;

use crate::airflow::model::common::DagRun;
use crate::app::events::custom::FlowrsEvent;
use crate::ui::common::create_headers;
use crate::ui::constants::{AirflowStateColor, ALTERNATING_ROW_COLOR, DEFAULT_STYLE, MARKED_COLOR};
use crate::ui::TIME_FORMAT;

use super::popup::commands_help::CommandPopUp;
use super::popup::dagruns::commands::DAGRUN_COMMAND_POP_UP;
use super::popup::dagruns::trigger::TriggerDagRunPopUp;
use super::popup::dagruns::DagRunPopUp;
use super::popup::error::ErrorPopup;
use super::popup::popup_area;
use super::popup::{dagruns::clear::ClearDagRunPopup, dagruns::mark::MarkDagRunPopup};
use super::{filter::Filter, Model, StatefulTable};
use crate::app::worker::{OpenItem, WorkerMessage};

pub struct DagRunModel {
    pub dag_id: Option<String>,
    pub dag_code: DagCodeWidget,
    pub all: Vec<DagRun>,
    pub filtered: StatefulTable<DagRun>,
    pub filter: Filter,
    pub marked: Vec<usize>,
    pub popup: Option<DagRunPopUp>,
    pub commands: Option<&'static CommandPopUp<'static>>,
    pub error_popup: Option<ErrorPopup>,
    ticks: u32,
    event_buffer: Vec<FlowrsEvent>,
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
        DagRunModel {
            dag_id: None,
            dag_code: DagCodeWidget::default(),
            all: vec![],
            filtered: StatefulTable::new(vec![]),
            filter: Filter::new(),
            marked: vec![],
            popup: None,
            commands: None,
            error_popup: None,
            ticks: 0,
            event_buffer: vec![],
        }
    }

    /// Calculate duration in seconds for a DAG run
    /// Returns None if `start_date` is not available
    fn calculate_duration(dag_run: &DagRun) -> Option<f64> {
        let start = dag_run.start_date?;
        let end = dag_run
            .end_date
            .unwrap_or_else(time::OffsetDateTime::now_utc);
        Some((end - start).as_seconds_f64())
    }

    /// Format duration as human-readable string (e.g., "2h 30m", "45s", "1d 3h")
    fn format_duration(seconds: f64) -> String {
        if seconds < 60.0 {
            format!("{seconds:.0}s")
        } else if seconds < 3600.0 {
            let minutes = (seconds / 60.0).floor();
            let secs = (seconds % 60.0).floor();
            if secs > 0.0 {
                format!("{minutes:.0}m {secs:.0}s")
            } else {
                format!("{minutes:.0}m")
            }
        } else if seconds < 86400.0 {
            let hours = (seconds / 3600.0).floor();
            let minutes = ((seconds % 3600.0) / 60.0).floor();
            if minutes > 0.0 {
                format!("{hours:.0}h {minutes:.0}m")
            } else {
                format!("{hours:.0}h")
            }
        } else {
            let days = (seconds / 86400.0).floor();
            let hours = ((seconds % 86400.0) / 3600.0).floor();
            if hours > 0.0 {
                format!("{days:.0}d {hours:.0}h")
            } else {
                format!("{days:.0}d")
            }
        }
    }

    /// Create a text-based duration gauge line
    /// The gauge will normalize durations to show relative progress within visible items
    fn create_duration_gauge(
        duration_seconds: f64,
        max_duration: f64,
        color: ratatui::style::Color,
        width: usize,
    ) -> Line<'static> {
        const FILLED_CHAR: &str = "─";
        const EMPTY_CHAR: &str = "─";

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

    pub fn filter_dag_runs(&mut self) {
        let prefix = self.filter.prefix();
        let mut filtered_dag_runs = match prefix {
            Some(prefix) => self
                .all
                .iter()
                .filter(|dagrun| dagrun.dag_run_id.contains(prefix))
                .cloned()
                .collect::<Vec<DagRun>>(),
            None => self.all.clone(),
        };
        // Sort by start_date in descending order (most recent first)
        filtered_dag_runs.sort_by(|a, b| b.start_date.cmp(&a.start_date));
        self.filtered.items = filtered_dag_runs;
    }

    fn update_autocomplete_candidates(&mut self) {
        // Get the search prefix from the filter's state machine
        let prefix = match self.filter.search_prefix() {
            Some(p) if !p.is_empty() => p,
            _ => {
                self.filter.set_autocomplete_candidates(Vec::new());
                return;
            }
        };

        // Get all dag_run_ids that start with the current prefix, sorted alphabetically
        let mut candidates: Vec<String> = self
            .all
            .iter()
            .map(|dagrun| dagrun.dag_run_id.clone())
            .filter(|dag_run_id| dag_run_id.starts_with(prefix))
            .collect();

        candidates.sort();
        candidates.dedup();

        self.filter.set_autocomplete_candidates(candidates);
    }

    pub fn current(&self) -> Option<&DagRun> {
        self.filtered
            .state
            .selected()
            .map(|i| &self.filtered.items[i])
    }

    pub fn mark_dag_run(&mut self, dag_run_id: &str, status: &str) {
        self.filtered.items.iter_mut().for_each(|dag_run| {
            if dag_run.dag_run_id == dag_run_id {
                dag_run.state = status.to_string();
            }
        });
    }
}

impl Default for DagRunModel {
    fn default() -> Self {
        Self::new()
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
                return (Some(FlowrsEvent::Tick), worker_messages);
            }
            FlowrsEvent::Key(key_event) => {
                if self.filter.is_enabled() {
                    let should_update_candidates = self.filter.update(key_event);
                    if should_update_candidates {
                        self.update_autocomplete_candidates();
                    }
                    self.filter_dag_runs();
                    return (None, vec![]);
                } else if let Some(_error_popup) = &mut self.error_popup {
                    match key_event.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            self.error_popup = None;
                        }
                        _ => (),
                    }
                    return (None, vec![]);
                } else if let Some(_commands) = &mut self.commands {
                    match key_event.code {
                        KeyCode::Char('q' | '?') | KeyCode::Esc => {
                            self.commands = None;
                            return (None, vec![]);
                        }
                        _ => (),
                    }
                } else if let Some(popup) = &mut self.popup {
                    // TODO: refactor this, should be all the same
                    match popup {
                        DagRunPopUp::Clear(popup) => {
                            let (key_event, messages) = popup.update(event);
                            debug!("Popup messages: {messages:?}");
                            if let Some(FlowrsEvent::Key(key_event)) = &key_event {
                                match key_event.code {
                                    KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => {
                                        self.popup = None;
                                    }
                                    _ => {}
                                }
                            }
                            return (None, messages);
                        }
                        DagRunPopUp::Mark(popup) => {
                            let (key_event, messages) = popup.update(event);
                            debug!("Popup messages: {messages:?}");
                            if let Some(FlowrsEvent::Key(key_event)) = &key_event {
                                match key_event.code {
                                    KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => {
                                        self.popup = None;
                                        self.marked = vec![];
                                    }
                                    _ => {}
                                }
                            }
                            return (None, messages);
                        }
                        DagRunPopUp::Trigger(popup) => {
                            let (key_event, messages) = popup.update(event);
                            debug!("Popup messages: {messages:?}");
                            if let Some(FlowrsEvent::Key(key_event)) = &key_event {
                                match key_event.code {
                                    KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => {
                                        self.popup = None;
                                    }
                                    _ => {}
                                }
                            }
                            return (None, messages);
                        }
                    }
                } else if self.dag_code.cached_lines.is_some() {
                    match key_event.code {
                        KeyCode::Esc | KeyCode::Char('q' | 'v') | KeyCode::Enter => {
                            self.dag_code.clear();
                            return (None, vec![]);
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            self.dag_code.vertical_scroll =
                                self.dag_code.vertical_scroll.saturating_add(1);
                            self.dag_code.vertical_scroll_state = self
                                .dag_code
                                .vertical_scroll_state
                                .position(self.dag_code.vertical_scroll);
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.dag_code.vertical_scroll =
                                self.dag_code.vertical_scroll.saturating_sub(1);
                            self.dag_code.vertical_scroll_state = self
                                .dag_code
                                .vertical_scroll_state
                                .position(self.dag_code.vertical_scroll);
                        }
                        _ => {}
                    }
                } else {
                    match key_event.code {
                        KeyCode::Down | KeyCode::Char('j') => {
                            self.filtered.next();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.filtered.previous();
                        }
                        KeyCode::Char('G') => {
                            self.filtered.state.select_last();
                        }
                        KeyCode::Char('g') => {
                            if let Some(FlowrsEvent::Key(key_event)) = self.event_buffer.pop() {
                                if key_event.code == KeyCode::Char('g') {
                                    self.filtered.state.select_first();
                                } else {
                                    self.event_buffer.push(FlowrsEvent::Key(key_event));
                                }
                            } else {
                                self.event_buffer.push(FlowrsEvent::Key(*key_event));
                            }
                        }
                        KeyCode::Char('t') => {
                            self.popup = Some(DagRunPopUp::Trigger(TriggerDagRunPopUp::new(
                                self.dag_id.clone().unwrap(),
                            )));
                        }
                        KeyCode::Char('m') => {
                            if let Some(index) = self.filtered.state.selected() {
                                self.marked.push(index);

                                self.popup = Some(DagRunPopUp::Mark(MarkDagRunPopup::new(
                                    self.marked
                                        .iter()
                                        .map(|i| self.filtered.items[*i].dag_run_id.clone())
                                        .collect(),
                                    self.current().unwrap().dag_id.clone(),
                                )));
                            }
                        }
                        KeyCode::Char('M') => {
                            if let Some(index) = self.filtered.state.selected() {
                                if self.marked.contains(&index) {
                                    self.marked.retain(|&i| i != index);
                                } else {
                                    self.marked.push(index);
                                }
                            }
                        }
                        KeyCode::Char('?') => {
                            self.commands = Some(&*DAGRUN_COMMAND_POP_UP);
                        }
                        KeyCode::Char('/') => {
                            self.filter.toggle();
                            self.filter_dag_runs();
                        }
                        KeyCode::Char('v') => {
                            if let Some(dag_id) = &self.dag_id {
                                return (
                                    None,
                                    vec![WorkerMessage::GetDagCode {
                                        dag_id: dag_id.clone(),
                                    }],
                                );
                            }
                        }
                        KeyCode::Char('c') => {
                            if let (Some(dag_run), Some(dag_id)) = (self.current(), &self.dag_id) {
                                self.popup = Some(DagRunPopUp::Clear(ClearDagRunPopup::new(
                                    dag_run.dag_run_id.clone(),
                                    dag_id.clone(),
                                )));
                            }
                        }
                        KeyCode::Enter => {
                            if let (Some(dag_id), Some(dag_run)) = (&self.dag_id, &self.current()) {
                                return (
                                    Some(FlowrsEvent::Key(*key_event)),
                                    vec![WorkerMessage::UpdateTaskInstances {
                                        dag_id: dag_id.clone(),
                                        dag_run_id: dag_run.dag_run_id.clone(),
                                        clear: true,
                                    }],
                                );
                            }
                        }
                        KeyCode::Char('o') => {
                            if let (Some(dag_id), Some(dag_run)) = (&self.dag_id, &self.current()) {
                                return (
                                    Some(FlowrsEvent::Key(*key_event)),
                                    vec![WorkerMessage::OpenItem(OpenItem::DagRun {
                                        dag_id: dag_id.clone(),
                                        dag_run_id: dag_run.dag_run_id.clone(),
                                    })],
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
            FlowrsEvent::Mouse => {}
        }
        (Some(event.clone()), vec![])
    }
}

impl Widget for &mut DagRunModel {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let rects = if self.filter.is_enabled() {
            let rects = Layout::default()
                .constraints([Constraint::Fill(90), Constraint::Max(3)].as_ref())
                .margin(0)
                .split(area);

            self.filter.render(rects[1], buf);
            rects
        } else {
            Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .margin(0)
                .split(area)
        };

        let headers = [
            "State",
            "Start Date",
            "Logical Date",
            "Trigger",
            "Duration",
            "Time",
        ];
        let header_row = create_headers(headers);
        let header =
            Row::new(header_row).style(DEFAULT_STYLE.reversed().add_modifier(Modifier::BOLD));

        // Calculate max duration for normalization
        let max_duration = self
            .filtered
            .items
            .iter()
            .filter_map(DagRunModel::calculate_duration)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(1.0);

        // Calculate the width available for the Duration column
        // Total width - borders(2) - state(5) - start_date(20) - logical_date(20) - trigger(11) - time(10) - column spacing(10)
        let table_inner_width = rects[0].width.saturating_sub(2); // Subtract borders
        let fixed_columns_width = 5 + 20 + 20 + 11 + 10 + 10; // State + Start Date + Logical Date + Trigger + Time + spacing between columns
        let gauge_width = table_inner_width
            .saturating_sub(fixed_columns_width)
            .max(10) as usize;

        let rows = self.filtered.items.iter().enumerate().map(|(idx, item)| {
            let state_color = match item.state.as_str() {
                "success" => AirflowStateColor::Success.into(),
                "running" => AirflowStateColor::Running.into(),
                "failed" => AirflowStateColor::Failed.into(),
                "queued" => AirflowStateColor::Queued.into(),
                _ => AirflowStateColor::None.into(),
            };

            let (duration_cell, time_cell) =
                if let Some(duration) = DagRunModel::calculate_duration(item) {
                    (
                        DagRunModel::create_duration_gauge(
                            duration,
                            max_duration,
                            state_color,
                            gauge_width,
                        ),
                        Line::from(DagRunModel::format_duration(duration)),
                    )
                } else {
                    (Line::from("-"), Line::from("-"))
                };

            Row::new(vec![
                Line::from(match item.state.as_str() {
                    "success" => {
                        Span::styled("■", Style::default().fg(AirflowStateColor::Success.into()))
                    }
                    "running" => {
                        Span::styled("■", DEFAULT_STYLE.fg(AirflowStateColor::Running.into()))
                    }
                    "failed" => {
                        Span::styled("■", DEFAULT_STYLE.fg(AirflowStateColor::Failed.into()))
                    }
                    "queued" => {
                        Span::styled("■", DEFAULT_STYLE.fg(AirflowStateColor::Queued.into()))
                    }
                    _ => Span::styled("■", DEFAULT_STYLE.fg(AirflowStateColor::None.into())),
                }),
                Line::from(if let Some(date) = item.start_date {
                    date.format(&format_description::parse(TIME_FORMAT).unwrap())
                        .unwrap()
                        .clone()
                } else {
                    "None".to_string()
                }),
                Line::from(if let Some(date) = item.logical_date {
                    date.format(&format_description::parse(TIME_FORMAT).unwrap())
                        .unwrap()
                        .clone()
                } else {
                    "None".to_string()
                }),
                Line::from(item.run_type.as_str()),
                duration_cell,
                time_cell,
            ])
            .style(if self.marked.contains(&idx) {
                DEFAULT_STYLE.bg(MARKED_COLOR)
            } else if (idx % 2) == 0 {
                DEFAULT_STYLE
            } else {
                DEFAULT_STYLE.bg(ALTERNATING_ROW_COLOR)
            })
        });
        let t = Table::new(
            rows,
            &[
                Constraint::Length(5),  // State
                Constraint::Length(20), // Start Date
                Constraint::Length(20), // Logical Date
                Constraint::Length(11), // Trigger type
                Constraint::Fill(1),    // Duration gauge (expands)
                Constraint::Length(10), // Time (formatted duration)
            ],
        )
        .header(header)
        .block(
            Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .title(if let Some(dag_id) = &self.dag_id {
                    format!("DAGRuns ({dag_id}) - press <?> to see available commands")
                } else {
                    "DAGRuns".to_string()
                })
                .style(DEFAULT_STYLE),
        )
        .row_highlight_style(DEFAULT_STYLE.reversed());
        StatefulWidget::render(t, rects[0], buf, &mut self.filtered.state);

        if let Some(cached_lines) = &self.dag_code.cached_lines {
            let area = popup_area(area, 60, 90);

            let popup = Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .title("DAG Code")
                .border_style(DEFAULT_STYLE)
                .style(DEFAULT_STYLE)
                .title_style(DEFAULT_STYLE.add_modifier(Modifier::BOLD));

            #[allow(clippy::cast_possible_truncation)]
            let code_text = Paragraph::new(cached_lines.clone())
                .block(popup)
                .style(DEFAULT_STYLE)
                .wrap(Wrap { trim: true })
                .scroll((self.dag_code.vertical_scroll as u16, 0));

            Clear.render(area, buf); //this clears out the background
            code_text.render(area, buf);

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            scrollbar.render(area, buf, &mut self.dag_code.vertical_scroll_state);
        }

        match &mut self.popup {
            Some(DagRunPopUp::Clear(popup)) => {
                popup.render(area, buf);
            }
            Some(DagRunPopUp::Mark(popup)) => {
                popup.render(area, buf);
            }
            Some(DagRunPopUp::Trigger(popup)) => {
                popup.render(area, buf);
            }
            _ => (),
        }

        if let Some(commands) = &self.commands {
            commands.render(area, buf);
        }

        if let Some(error_popup) = &self.error_popup {
            error_popup.render(area, buf);
        }
    }
}

fn code_to_lines(dag_code: &str) -> Vec<Line<'static>> {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let syntax = ps.find_syntax_by_extension("py").unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    let mut lines: Vec<Line<'static>> = vec![];
    for line in LinesWithEndings::from(dag_code) {
        // LinesWithEndings enables use of newlines mode
        let line_spans: Vec<Span<'static>> = h
            .highlight_line(line, &ps)
            .unwrap()
            .into_iter()
            .filter_map(|segment| into_span(segment).ok())
            .map(|span: Span| {
                // Convert borrowed span to owned span
                Span::styled(span.content.to_string(), span.style)
            })
            .collect();
        lines.push(Line::from(line_spans));
    }
    lines
}
