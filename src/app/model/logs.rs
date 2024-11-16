use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{
        Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
        Tabs, Widget, Wrap,
    },
};
use regex::Regex;

use crate::{
    airflow::model::log::Log,
    app::{error::FlowrsError, events::custom::FlowrsEvent, worker::WorkerMessage},
    ui::constants::DM_RGB,
};

use super::Model;

pub struct LogModel {
    pub dag_id: Option<String>,
    pub dag_run_id: Option<String>,
    pub task_id: Option<String>,
    pub tries: Option<u16>,
    pub all: Vec<Log>,
    pub current: usize,
    #[allow(dead_code)]
    pub errors: Vec<FlowrsError>,
    ticks: u32,
    vertical_scroll: usize,
    vertical_scroll_state: ScrollbarState,
}

impl LogModel {
    pub fn new() -> Self {
        LogModel {
            dag_id: None,
            dag_run_id: None,
            task_id: None,
            tries: None,
            all: vec![],
            current: 0,
            errors: vec![],
            ticks: 0,
            vertical_scroll: 0,
            vertical_scroll_state: ScrollbarState::default(),
        }
    }
}

impl Default for LogModel {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for LogModel {
    fn update(&mut self, event: &FlowrsEvent) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        match event {
            FlowrsEvent::Tick => {
                self.ticks += 1;
                if self.ticks % 10 != 0 {
                    return (Some(FlowrsEvent::Tick), vec![]);
                }
                if let (Some(dag_run_id), Some(dag_id), Some(task_id), Some(tries)) =
                    (&self.dag_run_id, &self.dag_id, &self.task_id, &self.tries)
                {
                    log::debug!("Updating task instances for dag_run_id: {}", dag_run_id);
                    return (
                        Some(FlowrsEvent::Tick),
                        vec![WorkerMessage::GetTaskLogs {
                            dag_id: dag_id.clone(),
                            dag_run_id: dag_run_id.clone(),
                            task_id: task_id.clone(),
                            task_try: *tries,
                        }],
                    );
                }
                return (Some(FlowrsEvent::Tick), vec![]);
            }
            FlowrsEvent::Key(key) => match key.code {
                KeyCode::Char('l') => {
                    self.current += 1;
                }
                KeyCode::Char('h') => {
                    if self.current > 0 {
                        self.current -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.vertical_scroll = self.vertical_scroll.saturating_add(1);
                    self.vertical_scroll_state =
                        self.vertical_scroll_state.position(self.vertical_scroll)
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
                    self.vertical_scroll_state =
                        self.vertical_scroll_state.position(self.vertical_scroll)
                }

                _ => return (Some(FlowrsEvent::Key(*key)), vec![]), // if no match, return the event
            },
            _ => (),
        }

        (None, vec![])
    }
}

impl Widget for &mut LogModel {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        if self.all.is_empty() {
            Paragraph::new("No logs available")
                .block(Block::default().borders(Borders::ALL).title("Logs"))
                .render(area, buffer);
            return;
        }

        let tab_titles = (0..self.all.len())
            .map(|i| format!("Task {}", i + 1))
            .collect::<Vec<String>>();

        let tabs = Tabs::new(tab_titles)
            .block(Block::default().title("Logs").borders(Borders::ALL))
            .select(self.current % self.all.len())
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().fg(DM_RGB));

        // Render the tabs
        tabs.render(area, buffer);

        // Define the layout for content under the tabs
        let chunks = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        if let Some(log) = self.all.get(self.current % self.all.len()) {
            let mut content = Text::default();
            let fragments = parse_content(&log.content);
            // This works but is extremely ugly. TODO: refactor, and test
            for (_, log_fragment) in fragments {
                let replaced_log = log_fragment.replace("\\n", "\n");
                let lines: Vec<String> = replaced_log.lines().map(|s| s.to_string()).collect();
                for line in lines {
                    content.push_line(Line::raw(line));
                }
            }

            let paragraph = Paragraph::new(content)
                .block(Block::default().borders(Borders::ALL).title("Content"))
                .wrap(Wrap { trim: true })
                .style(Style::default().fg(Color::White))
                .scroll((self.vertical_scroll as u16, 0));

            // Render the selected log's content
            paragraph.render(chunks[1], buffer);

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            scrollbar.render(chunks[1], buffer, &mut self.vertical_scroll_state);
        }
    }
}

// Log content is a list of tuples of form ('element1', 'element2'), i.e. serialized python tuples
fn parse_content(content: &str) -> Vec<(String, String)> {
    // Regex to match tuples of form ('element1', 'element2'). TODO: add tests, cause this is disgusting
    let re = Regex::new(r"\(\s*'((?:\\.|[^'])*)'\s*,\s*'((?:\\.|[^'])*)'\s*\)").unwrap();

    // Use regex to extract tuples
    re.captures_iter(content)
        .map(|cap| (cap[1].to_string(), cap[2].to_string()))
        .collect()
}
