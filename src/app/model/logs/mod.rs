mod render;
mod search;

use crossterm::event::KeyCode;
use ratatui::layout::Position;
use ratatui::widgets::ScrollbarState;

use crate::airflow::model::common::{Log, OpenItem};
use crate::app::events::custom::FlowrsEvent;
use crate::app::worker::WorkerMessage;

use search::{Search, SearchData};

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
    /// Scroll so the given source line is at the top of the viewport (used by
    /// search jumps). Since the wrapped position depends on the render width,
    /// this is resolved to a `Manual` position at render time.
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

pub struct LogModel {
    pub all: Vec<Log>,
    pub current: usize,
    pub error_popup: Option<ErrorPopup>,
    /// Terminal cursor position for the search bar; set during render while
    /// the search bar is open so `draw_ui` can place the cursor.
    pub search_cursor_position: Option<Position>,
    ticks: u32,
    poll_tick_multiplier: u32,
    pub(crate) scroll_mode: ScrollMode,
    pub(crate) vertical_scroll_state: ScrollbarState,
    pending_g: bool,
    pub(crate) search: Search,
}

impl Default for LogModel {
    fn default() -> Self {
        Self {
            all: Vec::new(),
            current: 0,
            error_popup: None,
            search_cursor_position: None,
            ticks: 0,
            poll_tick_multiplier: 10,
            scroll_mode: ScrollMode::default(),
            vertical_scroll_state: ScrollbarState::default(),
            pending_g: false,
            search: Search::default(),
        }
    }
}

impl LogModel {
    pub fn new(poll_tick_multiplier: u32) -> Self {
        Self {
            poll_tick_multiplier,
            ..Self::default()
        }
    }

    /// Reset scroll to follow mode (used when navigating to a new context)
    pub fn reset_scroll(&mut self) {
        self.scroll_mode = ScrollMode::Following;
    }

    /// Update the logs content. When in follow mode, the scroll position
    /// will automatically track the bottom at render time.
    pub fn update_logs(&mut self, logs: Vec<Log>) {
        self.all = logs;
        self.refresh_search();
    }

    /// Returns the total number of lines in the current log content
    pub(crate) fn current_line_count(&self) -> usize {
        let Some(log) = self.all.get(self.current % self.all.len().max(1)) else {
            return 0;
        };
        log.content.lines().count()
    }

    /// Recompute search matches against the current log tab's content.
    fn refresh_search(&mut self) {
        if self.all.is_empty() {
            self.search.refresh("");
        } else {
            let idx = self.current % self.all.len();
            self.search.refresh(&self.all[idx].content);
        }
    }

    /// Apply an edit to the query while the search bar is open, then jump to
    /// the first match (vim `incsearch` behavior).
    fn edit_search(&mut self, edit: impl FnOnce(&mut String)) {
        if let Search::Editing(data) = &mut self.search {
            edit(&mut data.query);
            data.current = 0;
        }
        self.refresh_search();
        self.scroll_to_current_match();
    }

    /// Confirm the typed query: keep highlights and hand navigation to `n`/`N`.
    fn confirm_search(&mut self) {
        self.search = match std::mem::take(&mut self.search) {
            Search::Editing(data) if !data.query.is_empty() => Search::Applied(data),
            _ => Search::Inactive,
        };
    }

    /// Move to the next/previous match, wrapping around.
    fn cycle_match(&mut self, forward: bool) {
        if let Search::Applied(data) = &mut self.search {
            data.cycle(forward);
        }
        self.scroll_to_current_match();
    }

    fn scroll_to_current_match(&mut self) {
        if let Some(m) = self.search.data().and_then(SearchData::current_match) {
            self.scroll_mode = ScrollMode::SourceLine { line: m.line };
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
                if !self.ticks.is_multiple_of(self.poll_tick_multiplier) {
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
                // While the search bar is open it captures all key input
                if self.search.is_editing() {
                    match key.code {
                        KeyCode::Enter => self.confirm_search(),
                        KeyCode::Esc => self.search = Search::Inactive,
                        KeyCode::Backspace => self.edit_search(|query| {
                            query.pop();
                        }),
                        KeyCode::Char(c) => self.edit_search(|query| query.push(c)),
                        _ => {}
                    }
                    return (None, vec![]);
                }
                // Clear pending 'g' on any key that is not 'g' to ensure gg requires consecutive presses
                if key.code != KeyCode::Char('g') {
                    self.pending_g = false;
                }
                match key.code {
                    KeyCode::Char('/') => {
                        self.search = Search::Editing(SearchData::default());
                    }
                    KeyCode::Char('n') if matches!(self.search, Search::Applied(_)) => {
                        self.cycle_match(true);
                    }
                    KeyCode::Char('N') if matches!(self.search, Search::Applied(_)) => {
                        self.cycle_match(false);
                    }
                    KeyCode::Esc if matches!(self.search, Search::Applied(_)) => {
                        // Clear search highlights (vim `:noh`)
                        self.search = Search::Inactive;
                    }
                    KeyCode::Char('l') | KeyCode::Right => {
                        if !self.all.is_empty() && self.current < self.all.len() - 1 {
                            self.current += 1;
                            self.refresh_search();
                        }
                    }
                    KeyCode::Char('h') | KeyCode::Left => {
                        if self.all.is_empty() || self.current == 0 {
                            // Navigate back to previous panel
                            return (Some(FlowrsEvent::Key(*key)), vec![]);
                        }
                        self.current -= 1;
                        self.refresh_search();
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

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyEvent, KeyModifiers};

    use crate::app::state::NavigationContext;

    use super::search::SearchMatch;
    use super::*;

    fn model_with_logs(contents: &[&str]) -> LogModel {
        let mut model = LogModel::default();
        model.update_logs(
            contents
                .iter()
                .map(|content| Log {
                    continuation_token: None,
                    content: (*content).to_string(),
                })
                .collect(),
        );
        model
    }

    fn press(model: &mut LogModel, code: KeyCode) -> Option<FlowrsEvent> {
        let event = FlowrsEvent::Key(KeyEvent::new(code, KeyModifiers::NONE));
        model.update(&event, &NavigationContext::None).0
    }

    fn type_query(model: &mut LogModel, query: &str) {
        press(model, KeyCode::Char('/'));
        for c in query.chars() {
            press(model, KeyCode::Char(c));
        }
    }

    fn current_match(model: &LogModel) -> Option<SearchMatch> {
        model.search.data().and_then(SearchData::current_match)
    }

    #[test]
    fn typing_highlights_and_jumps_to_first_match() {
        let mut model = model_with_logs(&["one\ntwo error\nthree error"]);
        type_query(&mut model, "error");

        assert!(model.search.is_editing());
        assert_eq!(model.search.data().unwrap().matches.len(), 2);
        // Incremental search jumps to the first match's source line
        assert!(matches!(
            model.scroll_mode,
            ScrollMode::SourceLine { line: 1 }
        ));
    }

    #[test]
    fn enter_applies_search_and_n_cycles_matches() {
        let mut model = model_with_logs(&["error\nok\nerror\nerror"]);
        type_query(&mut model, "error");
        press(&mut model, KeyCode::Enter);
        assert!(matches!(model.search, Search::Applied(_)));

        press(&mut model, KeyCode::Char('n'));
        assert_eq!(current_match(&model).unwrap().line, 2);
        press(&mut model, KeyCode::Char('n'));
        assert_eq!(current_match(&model).unwrap().line, 3);
        // Wraps around to the first match
        press(&mut model, KeyCode::Char('n'));
        assert_eq!(current_match(&model).unwrap().line, 0);
        // N goes backwards, wrapping to the last match
        press(&mut model, KeyCode::Char('N'));
        assert_eq!(current_match(&model).unwrap().line, 3);

        // A confirmed query without matches must survive n (no modulo-by-zero)
        type_query(&mut model, "no such text");
        press(&mut model, KeyCode::Enter);
        press(&mut model, KeyCode::Char('n'));
        assert_eq!(current_match(&model), None);
    }

    #[test]
    fn esc_cancels_input_and_clears_applied_search() {
        let mut model = model_with_logs(&["error"]);
        type_query(&mut model, "error");
        press(&mut model, KeyCode::Esc);
        assert!(matches!(model.search, Search::Inactive));

        type_query(&mut model, "error");
        press(&mut model, KeyCode::Enter);
        press(&mut model, KeyCode::Esc);
        assert!(matches!(model.search, Search::Inactive));
    }

    #[test]
    fn confirming_an_empty_query_deactivates_search() {
        let mut model = model_with_logs(&["error"]);
        press(&mut model, KeyCode::Char('/'));
        press(&mut model, KeyCode::Enter);
        assert!(matches!(model.search, Search::Inactive));
    }

    #[test]
    fn search_input_consumes_panel_and_global_keys() {
        let mut model = model_with_logs(&["quick brown fox"]);
        press(&mut model, KeyCode::Char('/'));
        // 'q' (global quit) and 'j'/'G' (scroll keys) must go into the query
        for code in ['q', 'u', 'i', 'G', 'j'] {
            assert!(press(&mut model, KeyCode::Char(code)).is_none());
        }
        assert_eq!(model.search.data().unwrap().query, "quiGj");
        // The only scroll movement was the incsearch jump while "q"/"qu"/"qui"
        // still matched; 'G' and 'j' did not act as scroll keys
        assert!(matches!(
            model.scroll_mode,
            ScrollMode::SourceLine { line: 0 }
        ));
    }

    #[test]
    fn log_refresh_keeps_search_and_recomputes_matches() {
        let mut model = model_with_logs(&["error"]);
        type_query(&mut model, "error");
        press(&mut model, KeyCode::Enter);

        model.update_logs(vec![Log {
            continuation_token: None,
            content: "error\nnew line with error".to_string(),
        }]);
        assert_eq!(model.search.data().unwrap().matches.len(), 2);
    }

    #[test]
    fn switching_tabs_recomputes_matches() {
        let mut model = model_with_logs(&["error error", "error"]);
        type_query(&mut model, "error");
        press(&mut model, KeyCode::Enter);
        assert_eq!(model.search.data().unwrap().matches.len(), 2);

        press(&mut model, KeyCode::Char('l'));
        assert_eq!(model.current, 1);
        assert_eq!(model.search.data().unwrap().matches.len(), 1);

        press(&mut model, KeyCode::Char('h'));
        assert_eq!(model.search.data().unwrap().matches.len(), 2);
    }

    #[test]
    fn n_falls_through_when_no_search_is_active() {
        let mut model = model_with_logs(&["error"]);
        assert!(press(&mut model, KeyCode::Char('n')).is_some());
    }
}
