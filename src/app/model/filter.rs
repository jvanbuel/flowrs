use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::{Color, Style, Styled},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

use crate::ui::constants::DEFAULT_STYLE;

#[derive(Clone)]
pub struct CursorState {
    pub position: Position,
}

/// Represents the state of the autocomplete system
#[derive(Debug, Clone, PartialEq)]
enum AutocompleteState {
    /// User is typing, no tab completion active
    Typing {
        prefix: String,
    },
    /// User has pressed Tab and is cycling through completions
    Cycling {
        /// The original prefix before any Tab completion
        original_prefix: String,
        /// Current accepted candidate
        current_prefix: String,
        /// Index of next candidate to show
        candidate_index: usize,
    },
    /// No input yet
    Empty,
}

impl AutocompleteState {
    fn current_prefix(&self) -> Option<&str> {
        match self {
            AutocompleteState::Typing { prefix } => Some(prefix.as_str()),
            AutocompleteState::Cycling { current_prefix, .. } => Some(current_prefix.as_str()),
            AutocompleteState::Empty => None,
        }
    }

    fn search_prefix(&self) -> Option<&str> {
        match self {
            AutocompleteState::Typing { prefix } => Some(prefix.as_str()),
            AutocompleteState::Cycling { original_prefix, .. } => Some(original_prefix.as_str()),
            AutocompleteState::Empty => None,
        }
    }

    fn candidate_index(&self) -> usize {
        match self {
            AutocompleteState::Cycling { candidate_index, .. } => *candidate_index,
            _ => 0,
        }
    }

    fn should_preserve_index(&self) -> bool {
        matches!(self, AutocompleteState::Cycling { .. })
    }
}

pub struct Filter {
    pub enabled: bool,
    autocomplete_state: AutocompleteState,
    pub cursor: CursorState,
    pub autocomplete_candidates: Vec<String>,
}

impl Filter {
    pub fn new() -> Filter {
        Filter {
            enabled: false,
            autocomplete_state: AutocompleteState::Empty,
            cursor: CursorState {
                position: Position::default(),
            },
            autocomplete_candidates: Vec::new(),
        }
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn prefix(&self) -> Option<&str> {
        self.autocomplete_state.current_prefix()
    }

    pub fn reset(&mut self) {
        self.enabled = false;
        self.autocomplete_state = AutocompleteState::Empty;
        self.autocomplete_candidates.clear();
    }

    pub fn set_autocomplete_candidates(&mut self, candidates: Vec<String>) {
        let preserve_index = self.autocomplete_state.should_preserve_index();
        let current_index = self.autocomplete_state.candidate_index();

        self.autocomplete_candidates = candidates;

        // Update the candidate index in the state if we're in Cycling mode
        if let AutocompleteState::Cycling { original_prefix, current_prefix, candidate_index } = &self.autocomplete_state {
            let new_index = if preserve_index && current_index < self.autocomplete_candidates.len() {
                current_index
            } else if !self.autocomplete_candidates.is_empty() {
                0
            } else {
                *candidate_index
            };

            self.autocomplete_state = AutocompleteState::Cycling {
                original_prefix: original_prefix.clone(),
                current_prefix: current_prefix.clone(),
                candidate_index: new_index,
            };
        }
    }

    pub fn get_autocomplete_suggestion(&self) -> Option<&str> {
        if self.autocomplete_candidates.is_empty() {
            return None;
        }

        let index = self.autocomplete_state.candidate_index();
        let current = self.autocomplete_candidates.get(index)?;
        let prefix = self.autocomplete_state.current_prefix()?;

        // Return the suffix that would complete the current prefix
        if current.starts_with(prefix) && current.len() > prefix.len() {
            Some(&current[prefix.len()..])
        } else {
            None
        }
    }

    pub fn update(&mut self, key_event: &KeyEvent) -> bool {
        match key_event.code {
            KeyCode::Esc | KeyCode::Enter => {
                self.toggle();
                false
            }
            KeyCode::Backspace => {
                self.autocomplete_state = match &self.autocomplete_state {
                    AutocompleteState::Empty => AutocompleteState::Empty,
                    AutocompleteState::Typing { prefix } | AutocompleteState::Cycling { current_prefix: prefix, .. } => {
                        let mut new_prefix = prefix.clone();
                        new_prefix.pop();
                        if new_prefix.is_empty() {
                            AutocompleteState::Empty
                        } else {
                            AutocompleteState::Typing { prefix: new_prefix }
                        }
                    }
                };
                true
            }
            KeyCode::Tab => {
                if self.autocomplete_candidates.is_empty() {
                    return false;
                }

                self.autocomplete_state = match &self.autocomplete_state {
                    AutocompleteState::Empty => AutocompleteState::Empty,
                    AutocompleteState::Typing { prefix } => {
                        // First Tab: transition to Cycling state
                        let candidate = self.autocomplete_candidates.first().cloned().unwrap_or_else(|| prefix.clone());
                        let next_index = usize::from(self.autocomplete_candidates.len() > 1);
                        AutocompleteState::Cycling {
                            original_prefix: prefix.clone(),
                            current_prefix: candidate,
                            candidate_index: next_index,
                        }
                    }
                    AutocompleteState::Cycling { original_prefix, candidate_index, .. } => {
                        // Subsequent Tab: cycle to next candidate
                        let next_index = (*candidate_index + 1) % self.autocomplete_candidates.len();
                        let candidate = self.autocomplete_candidates.get(*candidate_index).cloned().unwrap_or_else(|| original_prefix.clone());
                        AutocompleteState::Cycling {
                            original_prefix: original_prefix.clone(),
                            current_prefix: candidate,
                            candidate_index: next_index,
                        }
                    }
                };
                true
            }
            KeyCode::Char(c) => {
                self.autocomplete_state = match &self.autocomplete_state {
                    AutocompleteState::Empty => {
                        AutocompleteState::Typing { prefix: c.to_string() }
                    }
                    AutocompleteState::Typing { prefix } | AutocompleteState::Cycling { current_prefix: prefix, .. } => {
                        let mut new_prefix = prefix.clone();
                        new_prefix.push(c);
                        AutocompleteState::Typing { prefix: new_prefix }
                    }
                };
                true
            }
            _ => false,
        }
    }

    pub fn cursor_position(&self) -> &Position {
        &self.cursor.position
    }

    /// Returns the prefix to use for searching/filtering candidates
    /// This is the original prefix when cycling, or the current prefix when typing
    pub fn search_prefix(&self) -> Option<&str> {
        self.autocomplete_state.search_prefix()
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for &mut Filter {
    #[allow(clippy::cast_possible_truncation)]
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Get the text values we need first to avoid borrow issues
        let filter_text = self.prefix().unwrap_or("").to_string();
        let suggestion = self.get_autocomplete_suggestion().map(std::string::ToString::to_string);

        let filter_length = filter_text.len();
        self.cursor.position = Position {
            x: area.x + 1 + filter_length as u16,
            y: area.y + 1,
        };

        // Build the display text with autocomplete suggestion
        let display_line = if let Some(suggestion) = suggestion {
            Line::from(vec![
                Span::styled(filter_text, DEFAULT_STYLE),
                Span::styled(suggestion, Style::default().fg(Color::DarkGray)),
            ])
        } else {
            Line::from(filter_text)
        };

        let paragraph = Paragraph::new(display_line)
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .title("filter"),
            )
            .set_style(DEFAULT_STYLE);

        Widget::render(paragraph, area, buf);
    }
}
