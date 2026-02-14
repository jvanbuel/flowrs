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
    /// User is manually scrolling at a fixed position.
    Manual { position: usize },
}

impl ScrollMode {
    /// Returns the concrete scroll position for the given content length.
    fn position(&self, line_count: usize) -> usize {
        match self {
            ScrollMode::Following => line_count.saturating_sub(1),
            ScrollMode::Manual { position } => *position,
        }
    }

    fn is_following(&self) -> bool {
        matches!(self, ScrollMode::Following)
    }
}

#[derive(Default)]
pub struct LogModel {
    pub all: Vec<Log>,
    pub current: usize,
    pub error_popup: Option<ErrorPopup>,
    ticks: u32,
    scroll_mode: ScrollMode,
    vertical_scroll_state: ScrollbarState,
    pending_g: bool,
}

impl LogModel {
    pub fn new() -> Self {
        Self::default() // ScrollMode defaults to Following
    }

    /// Reset scroll to follow mode (used when navigating to a new context)
    pub fn reset_scroll(&mut self) {
        self.scroll_mode = ScrollMode::Following;
    }

    /// Update the logs content. When in follow mode, the scroll position
    /// will automatically track the bottom at render time.
    pub fn update_logs(&mut self, logs: Vec<Log>) {
        self.all = logs;
    }

    /// Returns the total number of lines in the current log content
    fn current_line_count(&self) -> usize {
        let Some(log) = self.all.get(self.current % self.all.len().max(1)) else {
            return 0;
        };
        log.content.lines().count()
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
                // Clear pending 'g' on any key that is not 'g' to ensure gg requires consecutive presses
                if key.code != KeyCode::Char('g') {
                    self.pending_g = false;
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
                        if self.all.get(self.current % self.all.len()).is_some() {
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
            for line in log.content.lines() {
                content.push_line(Line::raw(line));
            }

            let line_count = self.current_line_count();
            let scroll_pos = self.scroll_mode.position(line_count);
            self.vertical_scroll_state = self.vertical_scroll_state.position(scroll_pos);

            #[allow(clippy::cast_possible_truncation)]
            let paragraph = Paragraph::new(content)
                .block(
                    Block::default()
                        .border_type(BorderType::Plain)
                        .borders(Borders::ALL)
                        .title(" Content ")
                        .title_bottom(if self.scroll_mode.is_following() {
                            " [F]ollow: ON - auto-scrolling "
                        } else {
                            " [F]ollow: OFF - press G to resume "
                        })
                        .border_style(BORDER_STYLE)
                        .title_style(TITLE_STYLE),
                )
                .wrap(Wrap { trim: true })
                .style(DEFAULT_STYLE)
                .scroll((scroll_pos as u16, 0));

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
