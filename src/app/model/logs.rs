use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Position, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Tabs, Widget, Wrap,
    },
};

use crate::{
    airflow::model::common::Log,
    app::{
        events::custom::FlowrsEvent,
        worker::{OpenItem, WorkerMessage},
    },
    ui::theme::{
        ACCENT, BORDER_STYLE, DEFAULT_STYLE, SEARCH_CURRENT_LINE_STYLE, SEARCH_HIGHLIGHT_STYLE,
        TITLE_STYLE,
    },
};

use super::popup::error::ErrorPopup;
use super::Model;

/// Represents the log viewer's scroll behavior.
///
/// Eliminates the invalid state where `follow_mode = true` but `vertical_scroll`
/// points somewhere other than the bottom.
#[derive(Default)]
pub enum ScrollMode {
    /// Automatically scroll to bottom when new content arrives (tail mode).
    /// The scroll position is computed from the content length at render time.
    #[default]
    Following,
    /// User is manually scrolling at a fixed wrapped-line position.
    Manual { position: usize },
    /// Scroll to a source line index (used by search n/N).
    /// Converted to wrapped-line offset at render time.
    SourceLine { line: usize },
}

impl ScrollMode {
    /// Returns the concrete scroll position for the given content length.
    fn position(&self, line_count: usize) -> usize {
        match self {
            ScrollMode::Following => line_count.saturating_sub(1),
            ScrollMode::Manual { position } => *position,
            ScrollMode::SourceLine { line } => *line,
        }
    }

    fn is_following(&self) -> bool {
        matches!(self, ScrollMode::Following)
    }
}

#[derive(Default)]
pub enum SearchMode {
    #[default]
    Inactive,
    Input { query: String },
    Active {
        query: String,
        match_lines: Vec<usize>,
        current_match: usize,
    },
}

impl SearchMode {
    fn is_input(&self) -> bool {
        matches!(self, SearchMode::Input { .. })
    }

    fn query(&self) -> Option<&str> {
        match self {
            SearchMode::Active { query, .. } => Some(query.as_str()),
            _ => None,
        }
    }

    fn current_match_line(&self) -> Option<usize> {
        match self {
            SearchMode::Active {
                match_lines,
                current_match,
                ..
            } => match_lines.get(*current_match).copied(),
            _ => None,
        }
    }
}

#[derive(Default)]
pub struct LogModel {
    pub all: Vec<Log>,
    pub current: usize,
    pub error_popup: Option<ErrorPopup>,
    pub search_cursor_position: Option<Position>,
    ticks: u32,
    scroll_mode: ScrollMode,
    vertical_scroll_state: ScrollbarState,
    pending_g: bool,
    search: SearchMode,
}

impl LogModel {
    pub fn new() -> Self {
        Self::default()
    }

    fn current_log(&self) -> Option<&Log> {
        if self.all.is_empty() {
            return None;
        }
        self.all.get(self.current % self.all.len())
    }

    /// Reset scroll to follow mode (used when navigating to a new context)
    pub fn reset_scroll(&mut self) {
        self.scroll_mode = ScrollMode::Following;
    }

    /// Update the logs content. When in follow mode, the scroll position
    /// will automatically track the bottom at render time.
    pub fn update_logs(&mut self, logs: Vec<Log>) {
        self.all = logs;
        self.refresh_matches(false);
    }

    /// Returns the total number of lines in the current log content
    fn current_line_count(&self) -> usize {
        self.current_log()
            .map_or(0, |log| log.content.lines().count())
    }

    /// Recompute search matches for the current log tab.
    /// When `reset_position` is true, the current match resets to 0 (e.g. tab switch).
    /// When false, the current match is clamped to remain valid (e.g. log refresh).
    fn refresh_matches(&mut self, reset_position: bool) {
        if let SearchMode::Active {
            query,
            match_lines,
            current_match,
        } = &mut self.search
        {
            let idx = if self.all.is_empty() {
                return;
            } else {
                self.current % self.all.len()
            };
            *match_lines = compute_matches(&self.all[idx].content, query);
            if reset_position {
                *current_match = 0;
            } else if *current_match >= match_lines.len() {
                *current_match = match_lines.len().saturating_sub(1);
            }
        }
    }

    fn confirm_search(&mut self) {
        if let SearchMode::Input { query } = &self.search {
            let query = query.clone();
            if query.is_empty() {
                self.search = SearchMode::Inactive;
                return;
            }
            let match_lines = self
                .current_log()
                .map(|log| compute_matches(&log.content, &query))
                .unwrap_or_default();
            if let Some(&first) = match_lines.first() {
                self.scroll_mode = ScrollMode::SourceLine { line: first };
            }
            self.search = SearchMode::Active {
                query,
                match_lines,
                current_match: 0,
            };
        }
    }

    fn next_match(&mut self) {
        if let SearchMode::Active {
            match_lines,
            current_match,
            ..
        } = &mut self.search
        {
            if match_lines.is_empty() {
                return;
            }
            *current_match = (*current_match + 1) % match_lines.len();
            self.scroll_mode = ScrollMode::SourceLine {
                line: match_lines[*current_match],
            };
        }
    }

    fn prev_match(&mut self) {
        if let SearchMode::Active {
            match_lines,
            current_match,
            ..
        } = &mut self.search
        {
            if match_lines.is_empty() {
                return;
            }
            *current_match = if *current_match == 0 {
                match_lines.len() - 1
            } else {
                *current_match - 1
            };
            self.scroll_mode = ScrollMode::SourceLine {
                line: match_lines[*current_match],
            };
        }
    }
}

impl Model for LogModel {
    fn update(
        &mut self,
        event: &FlowrsEvent,
        ctx: &crate::app::state::NavigationContext,
    ) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        match event {
            FlowrsEvent::Tick => {
                self.ticks += 1;
                if !self.ticks.is_multiple_of(10) {
                    return (Some(FlowrsEvent::Tick), vec![]);
                }
                if let (Some(dag_id), Some(dag_run_id), Some(task_id), Some(task_try)) = (
                    ctx.dag_id(),
                    ctx.dag_run_id(),
                    ctx.task_id(),
                    ctx.task_try(),
                ) {
                    log::debug!("Updating task logs for dag_run_id: {dag_run_id}");
                    return (
                        Some(FlowrsEvent::Tick),
                        vec![WorkerMessage::UpdateTaskLogs {
                            dag_id: dag_id.clone(),
                            dag_run_id: dag_run_id.clone(),
                            task_id: task_id.clone(),
                            task_try,
                        }],
                    );
                }
                return (Some(FlowrsEvent::Tick), vec![]);
            }
            FlowrsEvent::Key(key) => {
                if let Some(_error_popup) = &mut self.error_popup {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            self.error_popup = None;
                        }
                        _ => (),
                    }
                    return (None, vec![]);
                }
                // Search input mode consumes all keys
                if let SearchMode::Input { query } = &mut self.search {
                    match key.code {
                        KeyCode::Enter => {
                            self.confirm_search();
                        }
                        KeyCode::Esc => {
                            self.search = SearchMode::Inactive;
                        }
                        KeyCode::Backspace => {
                            if query.is_empty() {
                                self.search = SearchMode::Inactive;
                            } else {
                                query.pop();
                            }
                        }
                        KeyCode::Char(c) => {
                            query.push(c);
                        }
                        _ => {}
                    }
                    return (None, vec![]);
                }

                if key.code != KeyCode::Char('g') {
                    self.pending_g = false;
                }
                match key.code {
                    KeyCode::Char('/') => {
                        self.search = SearchMode::Input {
                            query: String::new(),
                        };
                    }
                    KeyCode::Char('n') => {
                        self.next_match();
                    }
                    KeyCode::Char('N') => {
                        self.prev_match();
                    }
                    KeyCode::Char('l') | KeyCode::Right => {
                        if !self.all.is_empty() && self.current < self.all.len() - 1 {
                            self.current += 1;
                            self.refresh_matches(true);
                        }
                    }
                    KeyCode::Char('h') | KeyCode::Left => {
                        if self.all.is_empty() || self.current == 0 {
                            // Navigate back to previous panel
                            return (Some(FlowrsEvent::Key(*key)), vec![]);
                        }
                        self.current -= 1;
                        self.refresh_matches(true);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let line_count = self.current_line_count();
                        let current_pos = self.scroll_mode.position(line_count);
                        let new_pos = current_pos.saturating_add(1);
                        if new_pos >= line_count.saturating_sub(1) {
                            self.scroll_mode = ScrollMode::Following;
                        } else {
                            self.scroll_mode = ScrollMode::Manual { position: new_pos };
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        let line_count = self.current_line_count();
                        let current_pos = self.scroll_mode.position(line_count);
                        self.scroll_mode = ScrollMode::Manual {
                            position: current_pos.saturating_sub(1),
                        };
                    }
                    KeyCode::Char('o') => {
                        if self.current_log().is_some() {
                            if let (Some(dag_id), Some(dag_run_id), Some(task_id)) =
                                (ctx.dag_id(), ctx.dag_run_id(), ctx.task_id())
                            {
                                return (
                                    Some(FlowrsEvent::Key(*key)),
                                    vec![WorkerMessage::OpenItem(OpenItem::Log {
                                        dag_id: dag_id.clone(),
                                        dag_run_id: dag_run_id.clone(),
                                        task_id: task_id.clone(),
                                        #[allow(clippy::cast_possible_truncation)]
                                        task_try: (self.current + 1) as u32,
                                    })],
                                );
                            }
                        }
                    }
                    KeyCode::Char('G') => {
                        self.scroll_mode = ScrollMode::Following;
                    }
                    KeyCode::Char('F') => {
                        // Toggle follow mode
                        if self.scroll_mode.is_following() {
                            let line_count = self.current_line_count();
                            self.scroll_mode = ScrollMode::Manual {
                                position: line_count.saturating_sub(1),
                            };
                        } else {
                            self.scroll_mode = ScrollMode::Following;
                        }
                    }
                    KeyCode::Char('g') => {
                        // gg: go to top of log
                        if self.pending_g {
                            self.scroll_mode = ScrollMode::Manual { position: 0 };
                            self.pending_g = false;
                        } else {
                            self.pending_g = true;
                        }
                    }
                    KeyCode::Esc => {
                        if matches!(self.search, SearchMode::Active { .. }) {
                            self.search = SearchMode::Inactive;
                        } else {
                            return (Some(FlowrsEvent::Key(*key)), vec![]);
                        }
                    }
                    _ => return (Some(FlowrsEvent::Key(*key)), vec![]), // if no match, return the event
                }
            }
            FlowrsEvent::Mouse | FlowrsEvent::FocusGained | FlowrsEvent::FocusLost => (),
        }

        (None, vec![])
    }
}

fn compute_matches(content: &str, query: &str) -> Vec<usize> {
    if query.is_empty() {
        return vec![];
    }
    let query_lower = query.to_lowercase();
    content
        .lines()
        .enumerate()
        .filter(|(_, line)| line.to_lowercase().contains(&query_lower))
        .map(|(i, _)| i)
        .collect()
}

fn highlight_line(line: &str, query: &str, is_current: bool) -> Line<'static> {
    let lower_query = query.to_lowercase();
    let match_style = if is_current {
        SEARCH_CURRENT_LINE_STYLE
    } else {
        SEARCH_HIGHLIGHT_STYLE
    };
    let text_style = if is_current {
        SEARCH_CURRENT_LINE_STYLE
    } else {
        DEFAULT_STYLE
    };

    // Build a mapping from char index to byte offset in the original line,
    // and a lowercased char vector for safe case-insensitive matching.
    let chars: Vec<char> = line.chars().collect();
    let lower_chars: Vec<char> = chars.iter().flat_map(|c| c.to_lowercase()).collect();
    let query_chars: Vec<char> = lower_query.chars().collect();

    // Find all char-index matches of the query in the lowercased char sequence.
    let mut match_ranges: Vec<(usize, usize)> = Vec::new();
    if !query_chars.is_empty() {
        let mut i = 0;
        while i + query_chars.len() <= lower_chars.len() {
            if lower_chars[i..i + query_chars.len()] == query_chars[..] {
                match_ranges.push((i, i + query_chars.len()));
                i += query_chars.len(); // non-overlapping
            } else {
                i += 1;
            }
        }
    }

    if match_ranges.is_empty() {
        return Line::styled(line.to_string(), text_style);
    }

    // Map lowercased char indices back to byte offsets in the original string.
    // Each original char may map to 1+ lowercased chars.
    let mut orig_to_lower: Vec<usize> = Vec::with_capacity(chars.len() + 1);
    let mut lower_idx = 0;
    for &c in &chars {
        orig_to_lower.push(lower_idx);
        lower_idx += c.to_lowercase().count();
    }
    orig_to_lower.push(lower_idx); // sentinel for end

    // Build byte-offset pairs in the original string for each match.
    let byte_offsets: Vec<usize> = chars
        .iter()
        .scan(0usize, |offset, c| {
            let current = *offset;
            *offset += c.len_utf8();
            Some(current)
        })
        .collect();
    let line_byte_len = line.len();

    let orig_char_for_lower = |lower_idx: usize| -> usize {
        match orig_to_lower.binary_search(&lower_idx) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        }
    };

    let mut spans = Vec::new();
    let mut last_orig_byte = 0;
    for &(lo_start, lo_end) in &match_ranges {
        let orig_start = orig_char_for_lower(lo_start);
        let orig_end = orig_char_for_lower(lo_end);
        let byte_start = byte_offsets.get(orig_start).copied().unwrap_or(line_byte_len);
        let byte_end = byte_offsets.get(orig_end).copied().unwrap_or(line_byte_len);

        if byte_start > last_orig_byte {
            spans.push(Span::styled(
                line[last_orig_byte..byte_start].to_string(),
                text_style,
            ));
        }
        spans.push(Span::styled(
            line[byte_start..byte_end].to_string(),
            match_style,
        ));
        last_orig_byte = byte_end;
    }
    if last_orig_byte < line_byte_len {
        spans.push(Span::styled(
            line[last_orig_byte..].to_string(),
            text_style,
        ));
    }
    Line::from(spans)
}

/// Number of visual (wrapped) rows a single line occupies at the given width.
fn wrapped_line_count(line_byte_len: usize, wrap_width: usize) -> usize {
    if line_byte_len == 0 {
        1
    } else {
        line_byte_len.div_ceil(wrap_width)
    }
}

fn source_line_to_wrapped(content: &str, source_line: usize, wrap_width: usize) -> usize {
    if wrap_width == 0 {
        return source_line;
    }
    content
        .lines()
        .take(source_line)
        .map(|line| wrapped_line_count(line.len(), wrap_width))
        .sum()
}

fn total_wrapped_lines(content: &str, wrap_width: usize) -> usize {
    if wrap_width == 0 {
        return content.lines().count();
    }
    content
        .lines()
        .map(|line| wrapped_line_count(line.len(), wrap_width))
        .sum()
}

fn build_highlighted_content(
    content: &str,
    query: &str,
    current_match_line: Option<usize>,
) -> Text<'static> {
    let mut text = Text::default();
    for (i, line_str) in content.lines().enumerate() {
        let is_current = current_match_line == Some(i);
        text.push_line(highlight_line(line_str, query, is_current));
    }
    text
}

impl Widget for &mut LogModel {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        self.search_cursor_position = None;

        if self.all.is_empty() {
            Paragraph::new("No logs available")
                .style(DEFAULT_STYLE)
                .block(
                    Block::default()
                        .border_type(BorderType::Rounded)
                        .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                        .border_style(BORDER_STYLE),
                )
                .render(area, buffer);
            return;
        }

        let tab_titles = (0..self.all.len())
            .map(|i| format!("Task {}", i + 1))
            .collect::<Vec<String>>();

        let tabs = Tabs::new(tab_titles)
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                    .border_style(BORDER_STYLE),
            )
            .select(self.current % self.all.len())
            .highlight_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
            .style(DEFAULT_STYLE);

        // Render the tabs
        tabs.render(area, buffer);

        // Define the layout for content under the tabs
        let show_search_bar = self.search.is_input();
        let chunks = if show_search_bar {
            Layout::default()
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(1),
                ])
                .split(area)
        } else {
            Layout::default()
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(area)
        };

        if let Some(log) = self.all.get(self.current % self.all.len()) {
            // Content area inner width (subtract 2 for left+right borders, 1 for scrollbar)
            let inner_width = chunks[1].width.saturating_sub(3) as usize;
            let wrapped_total = total_wrapped_lines(&log.content, inner_width);

            // SourceLine needs conversion to wrapped offset; normalize to Manual
            // so subsequent j/k presses use the correct coordinate space.
            let scroll_pos = match &self.scroll_mode {
                ScrollMode::Following => wrapped_total.saturating_sub(1),
                ScrollMode::Manual { position } => *position,
                ScrollMode::SourceLine { line } => {
                    let pos = source_line_to_wrapped(&log.content, *line, inner_width);
                    self.scroll_mode = ScrollMode::Manual { position: pos };
                    pos
                }
            };
            self.vertical_scroll_state = self
                .vertical_scroll_state
                .content_length(wrapped_total)
                .position(scroll_pos);

            let bottom_title = match &self.search {
                SearchMode::Active {
                    match_lines,
                    current_match,
                    ..
                } => {
                    if match_lines.is_empty() {
                        " [0/0] matches | / search | Esc clear ".to_string()
                    } else {
                        format!(
                            " [{}/{}] matches | n/N navigate | Esc clear ",
                            current_match + 1,
                            match_lines.len()
                        )
                    }
                }
                _ => {
                    if self.scroll_mode.is_following() {
                        " [F]ollow: ON - auto-scrolling | / search ".to_string()
                    } else {
                        " [F]ollow: OFF - press G to resume | / search ".to_string()
                    }
                }
            };

            // Only build highlighted text when search is active; otherwise use
            // a zero-allocation borrowed Text from the raw content.
            let owned_content;
            let content: Text<'_> = if let Some(q) = self.search.query() {
                let current_match_line = self.search.current_match_line();
                owned_content = build_highlighted_content(&log.content, q, current_match_line);
                owned_content
            } else {
                Text::raw(&log.content)
            };

            #[allow(clippy::cast_possible_truncation)]
            let paragraph = Paragraph::new(content)
                .block(
                    Block::default()
                        .border_type(BorderType::Plain)
                        .borders(Borders::ALL)
                        .title(" Content ")
                        .title_bottom(Line::raw(bottom_title))
                        .border_style(BORDER_STYLE)
                        .title_style(TITLE_STYLE),
                )
                .wrap(Wrap { trim: false })
                .style(DEFAULT_STYLE)
                .scroll((scroll_pos as u16, 0));

            // Render the selected log's content
            paragraph.render(chunks[1], buffer);

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            scrollbar.render(chunks[1], buffer, &mut self.vertical_scroll_state);
        }

        if let SearchMode::Input { query } = &self.search {
            let search_area = chunks[2];
            let search_line = Line::from(vec![
                Span::styled("/", Style::default().fg(ACCENT)),
                Span::styled(query.clone(), DEFAULT_STYLE),
            ]);
            Paragraph::new(search_line)
                .style(DEFAULT_STYLE)
                .render(search_area, buffer);

            #[allow(clippy::cast_possible_truncation)]
            {
                self.search_cursor_position = Some(Position {
                    x: search_area.x + 1 + query.len() as u16,
                    y: search_area.y,
                });
            }
        }

        if let Some(error_popup) = &self.error_popup {
            error_popup.render(area, buffer);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_matches_empty_query() {
        let empty: Vec<usize> = vec![];
        assert_eq!(compute_matches("hello\nworld", ""), empty);
    }

    #[test]
    fn test_compute_matches_no_match() {
        let empty: Vec<usize> = vec![];
        assert_eq!(compute_matches("hello\nworld", "foo"), empty);
    }

    #[test]
    fn test_compute_matches_single_match() {
        assert_eq!(compute_matches("hello\nworld\nfoo", "world"), vec![1]);
    }

    #[test]
    fn test_compute_matches_multiple_matches() {
        assert_eq!(
            compute_matches("error here\nok\nerror there", "error"),
            vec![0, 2]
        );
    }

    #[test]
    fn test_compute_matches_case_insensitive() {
        assert_eq!(
            compute_matches("ERROR here\nok\nError there", "error"),
            vec![0, 2]
        );
    }

    #[test]
    fn test_highlight_line_no_match() {
        let line = highlight_line("hello world", "foo", false);
        assert_eq!(line.spans.len(), 1);
        assert_eq!(line.spans[0].content, "hello world");
    }

    #[test]
    fn test_highlight_line_single_match() {
        let line = highlight_line("hello world", "world", false);
        assert_eq!(line.spans.len(), 2);
        assert_eq!(line.spans[0].content, "hello ");
        assert_eq!(line.spans[1].content, "world");
        assert_eq!(line.spans[1].style, SEARCH_HIGHLIGHT_STYLE);
    }

    #[test]
    fn test_highlight_line_current_match() {
        let line = highlight_line("hello world", "world", true);
        assert_eq!(line.spans.len(), 2);
        assert_eq!(line.spans[0].content, "hello ");
        assert_eq!(line.spans[0].style, SEARCH_CURRENT_LINE_STYLE);
        assert_eq!(line.spans[1].content, "world");
        assert_eq!(line.spans[1].style, SEARCH_CURRENT_LINE_STYLE);
    }

    #[test]
    fn test_highlight_line_case_insensitive() {
        let line = highlight_line("Hello WORLD", "world", false);
        assert_eq!(line.spans.len(), 2);
        assert_eq!(line.spans[0].content, "Hello ");
        assert_eq!(line.spans[1].content, "WORLD");
        assert_eq!(line.spans[1].style, SEARCH_HIGHLIGHT_STYLE);
    }

    #[test]
    fn test_highlight_line_multiple_matches() {
        let line = highlight_line("error and error", "error", false);
        assert_eq!(line.spans.len(), 3);
        assert_eq!(line.spans[0].content, "error");
        assert_eq!(line.spans[0].style, SEARCH_HIGHLIGHT_STYLE);
        assert_eq!(line.spans[1].content, " and ");
        assert_eq!(line.spans[2].content, "error");
        assert_eq!(line.spans[2].style, SEARCH_HIGHLIGHT_STYLE);
    }

    #[test]
    fn test_source_line_to_wrapped() {
        assert_eq!(source_line_to_wrapped("short\nshort", 1, 80), 1);
        // A 160-char line wraps to 2 lines at width 80
        let long = "x".repeat(160);
        let content = format!("{long}\nshort");
        assert_eq!(source_line_to_wrapped(&content, 1, 80), 2);
    }

    #[test]
    fn test_search_mode_transitions() {
        let mut model = LogModel::new();
        model.all = vec![Log {
            content: "line one error\nline two\nline three error".to_string(),
            continuation_token: None,
        }];

        assert!(matches!(model.search, SearchMode::Inactive));

        model.search = SearchMode::Input {
            query: "error".to_string(),
        };
        assert!(model.search.is_input());

        model.confirm_search();
        match &model.search {
            SearchMode::Active {
                query,
                match_lines,
                current_match,
            } => {
                assert_eq!(query, "error");
                assert_eq!(match_lines, &vec![0, 2]);
                assert_eq!(*current_match, 0);
            }
            _ => panic!("Expected Active search mode"),
        }

        model.next_match();
        if let SearchMode::Active { current_match, .. } = &model.search {
            assert_eq!(*current_match, 1);
        }

        model.next_match();
        if let SearchMode::Active { current_match, .. } = &model.search {
            assert_eq!(*current_match, 0);
        }

        model.prev_match();
        if let SearchMode::Active { current_match, .. } = &model.search {
            assert_eq!(*current_match, 1);
        }
    }
}
