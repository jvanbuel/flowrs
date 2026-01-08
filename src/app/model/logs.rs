use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Text},
    widgets::{
        Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Tabs, Widget, Wrap,
    },
};
use regex::Regex;

use crate::{
    airflow::model::common::Log,
    app::{
        events::custom::FlowrsEvent,
        worker::{OpenItem, WorkerMessage},
    },
    ui::theme::{ACCENT, BORDER_STYLE, DEFAULT_STYLE, TITLE_STYLE},
};

use super::popup::error::ErrorPopup;
use super::Model;

#[derive(Default)]
pub struct LogModel {
    pub dag_id: Option<String>,
    pub dag_run_id: Option<String>,
    pub task_id: Option<String>,
    pub tries: Option<u16>,
    pub all: Vec<Log>,
    pub current: usize,
    pub error_popup: Option<ErrorPopup>,
    ticks: u32,
    vertical_scroll: usize,
    vertical_scroll_state: ScrollbarState,
    event_buffer: Vec<KeyCode>,
}

impl LogModel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the total number of lines in the current log content
    fn current_line_count(&self) -> usize {
        let Some(log) = self.all.get(self.current % self.all.len().max(1)) else {
            return 0;
        };

        let fragments = parse_content(&log.content);
        if fragments.is_empty() {
            // v2 format: content is already a clean string
            log.content.lines().count()
        } else {
            // v1 format: parse Python tuples
            let mut count = 0;
            for (_, log_fragment) in fragments {
                let replaced_log = log_fragment.replace("\\n", "\n");
                count += replaced_log.lines().count();
            }
            count
        }
    }
}

impl Model for LogModel {
    fn update(&mut self, event: &FlowrsEvent) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        match event {
            FlowrsEvent::Tick => {
                self.ticks += 1;
                if !self.ticks.is_multiple_of(10) {
                    return (Some(FlowrsEvent::Tick), vec![]);
                }
                if let (Some(dag_run_id), Some(dag_id), Some(task_id), Some(tries)) =
                    (&self.dag_run_id, &self.dag_id, &self.task_id, &self.tries)
                {
                    log::debug!("Updating task instances for dag_run_id: {dag_run_id}");
                    return (
                        Some(FlowrsEvent::Tick),
                        vec![WorkerMessage::UpdateTaskLogs {
                            dag_id: dag_id.clone(),
                            dag_run_id: dag_run_id.clone(),
                            task_id: task_id.clone(),
                            task_try: *tries,
                            clear: false,
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
                match key.code {
                    KeyCode::Char('l') | KeyCode::Right => {
                        if !self.all.is_empty() && self.current < self.all.len() - 1 {
                            self.current += 1;
                        }
                    }
                    KeyCode::Char('h') | KeyCode::Left => {
                        if self.all.is_empty() || self.current == 0 {
                            // Navigate back to previous panel
                            return (Some(FlowrsEvent::Key(*key)), vec![]);
                        }
                        self.current -= 1;
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.vertical_scroll = self.vertical_scroll.saturating_add(1);
                        self.vertical_scroll_state =
                            self.vertical_scroll_state.position(self.vertical_scroll);
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
                        self.vertical_scroll_state =
                            self.vertical_scroll_state.position(self.vertical_scroll);
                    }
                    KeyCode::Char('o') => {
                        if self.all.get(self.current % self.all.len()).is_some() {
                            return (
                                Some(FlowrsEvent::Key(*key)),
                                vec![WorkerMessage::OpenItem(OpenItem::Log {
                                    dag_id: self.dag_id.clone().expect("DAG ID not set"),
                                    dag_run_id: self
                                        .dag_run_id
                                        .clone()
                                        .expect("DAG Run ID not set"),
                                    task_id: self.task_id.clone().expect("Task ID not set"),
                                    #[allow(clippy::cast_possible_truncation)]
                                    task_try: (self.current + 1) as u16,
                                })],
                            );
                        }
                    }
                    KeyCode::Char('G') => {
                        // Go to bottom of log
                        let line_count = self.current_line_count();
                        self.vertical_scroll = line_count.saturating_sub(1);
                        self.vertical_scroll_state =
                            self.vertical_scroll_state.position(self.vertical_scroll);
                    }
                    KeyCode::Char('g') => {
                        // gg: go to top of log
                        if let Some(last_key) = self.event_buffer.pop() {
                            if last_key == KeyCode::Char('g') {
                                self.vertical_scroll = 0;
                                self.vertical_scroll_state =
                                    self.vertical_scroll_state.position(self.vertical_scroll);
                            } else {
                                self.event_buffer.push(last_key);
                                self.event_buffer.push(KeyCode::Char('g'));
                            }
                        } else {
                            self.event_buffer.push(KeyCode::Char('g'));
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

impl Widget for &mut LogModel {
    fn render(self, area: Rect, buffer: &mut Buffer) {
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
        let chunks = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        if let Some(log) = self.all.get(self.current % self.all.len()) {
            let mut content = Text::default();
            let fragments = parse_content(&log.content);

            // If fragments is empty, content is likely v2 format (already a clean string)
            // Otherwise, it's v1 format with Python tuples that need parsing
            if fragments.is_empty() {
                // v2 format: content is already a clean string, just split by lines
                for line in log.content.lines() {
                    content.push_line(Line::raw(line));
                }
            } else {
                // v1 format: parse Python tuples
                for (_, log_fragment) in fragments {
                    let replaced_log = log_fragment.replace("\\n", "\n");
                    let lines: Vec<String> = replaced_log
                        .lines()
                        .map(std::string::ToString::to_string)
                        .collect();
                    for line in lines {
                        content.push_line(Line::raw(line));
                    }
                }
            }

            #[allow(clippy::cast_possible_truncation)]
            let paragraph = Paragraph::new(content)
                .block(
                    Block::default()
                        .border_type(BorderType::Plain)
                        .borders(Borders::ALL)
                        .title(" Content ")
                        .border_style(BORDER_STYLE)
                        .title_style(TITLE_STYLE),
                )
                .wrap(Wrap { trim: true })
                .style(DEFAULT_STYLE)
                .scroll((self.vertical_scroll as u16, 0));

            // Render the selected log's content
            paragraph.render(chunks[1], buffer);

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            scrollbar.render(chunks[1], buffer, &mut self.vertical_scroll_state);
        }

        if let Some(error_popup) = &self.error_popup {
            error_popup.render(area, buffer);
        }
    }
}

// Log content is a list of tuples of form ('element1', 'element2'), i.e. serialized python tuples
// The second element can be single or double quoted depending on content
fn parse_content(content: &str) -> Vec<(String, String)> {
    // Regex to match tuples with either quote style for second element:
    // ('element1', 'element2') OR ('element1', "element2")
    let re =
        Regex::new(r#"\(\s*'((?:\\.|[^'])*)'\s*,\s*(?:"((?:\\.|[^"])*)"|'((?:\\.|[^'])*)')\s*\)"#)
            .expect("Log parsing regex pattern should be valid");

    // Use regex to extract tuples
    re.captures_iter(content)
        .map(|cap| {
            let first = cap[1].to_string();
            // Second element can be in group 2 (double quotes) or group 3 (single quotes)
            let second = cap
                .get(2)
                .or_else(|| cap.get(3))
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            (first, second)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_content_single_quotes() {
        let content = "[('host1', 'log content here')]";
        let result = parse_content(content);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "host1");
        assert_eq!(result[0].1, "log content here");
    }

    #[test]
    fn test_parse_content_double_quotes_second_element() {
        let content = r#"[('cec849a302e3', "*** Found local files:\n***   * /opt/airflow/logs/")]"#;
        let result = parse_content(content);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "cec849a302e3");
        assert_eq!(
            result[0].1,
            r"*** Found local files:\n***   * /opt/airflow/logs/"
        );
    }

    #[test]
    fn test_parse_content_with_escaped_quotes() {
        let content = r"[('host', 'line with \' escaped quote')]";
        let result = parse_content(content);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "host");
        assert_eq!(result[0].1, r"line with \' escaped quote");
    }

    #[test]
    fn test_parse_content_multiple_tuples() {
        let content = r#"[('host1', 'log1'), ('host2', "log2 with special chars")]"#;
        let result = parse_content(content);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("host1".to_string(), "log1".to_string()));
        assert_eq!(
            result[1],
            ("host2".to_string(), "log2 with special chars".to_string())
        );
    }

    #[test]
    fn test_parse_content_real_airflow_log() {
        let content = r#"[('cec849a302e3', "*** Found local files:\\n***   * /opt/airflow/logs/dag_id=dataset_consumes_1/run_id=dataset_triggered__2025-10-12T01:24:15.313731+00:00/task_id=consuming_1/attempt=1.log\\n[2025-10-12T01:24:16.754+0000] {local_task_job_runner.py:123} INFO - ::group::Pre task execution logs")]"#;
        let result = parse_content(content);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "cec849a302e3");
        assert!(result[0].1.starts_with("*** Found local files:"));
    }
}
