/// Autocomplete state for filter input fields
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AutocompleteState {
    /// The current text in the input field
    pub typed: String,
    /// Filtered candidates that match the typed prefix
    pub candidates: Vec<String>,
    /// Index of the currently selected candidate (for Tab cycling)
    selected_index: usize,
    /// Original typed text before Tab completion (for reverting on backspace)
    original_typed: Option<String>,
}

impl AutocompleteState {
    /// Create an `AutocompleteState` with initial typed text and candidates (for tests)
    pub fn with_typed(typed: impl Into<String>, candidates: Vec<String>) -> Self {
        Self {
            typed: typed.into(),
            candidates,
            selected_index: 0,
            original_typed: None,
        }
    }

    /// Create an `AutocompleteState` with typed text and filtered candidates from available options
    pub fn with_typed_and_candidates(typed: impl Into<String>, available: &[String]) -> Self {
        let mut state = Self {
            typed: typed.into(),
            ..Default::default()
        };
        state.update_candidates(available);
        state
    }

    /// Returns the suffix to display as ghost text (grayed out).
    /// Always shows the completion for the first match.
    pub fn ghost_suffix(&self) -> Option<&str> {
        self.candidates
            .first()
            .and_then(|c| c.strip_prefix(&self.typed))
    }

    /// Accept Tab completion: set typed to current candidate, cycle to next.
    /// Returns true if a completion was accepted.
    pub fn accept_completion(&mut self) -> bool {
        if let Some(completed) = self.candidates.get(self.selected_index).cloned() {
            if self.original_typed.is_none() {
                self.original_typed = Some(self.typed.clone());
            }
            self.typed = completed;
            self.selected_index = (self.selected_index + 1) % self.candidates.len();
            true
        } else {
            false
        }
    }

    /// Accept Shift+Tab completion: set typed to previous candidate.
    /// Returns true if a completion was accepted.
    pub fn accept_prev_completion(&mut self) -> bool {
        if self.candidates.is_empty() {
            return false;
        }
        if self.original_typed.is_none() {
            self.original_typed = Some(self.typed.clone());
        }
        // Go to previous candidate
        self.selected_index = if self.selected_index == 0 {
            self.candidates.len() - 1
        } else {
            self.selected_index - 1
        };
        if let Some(completed) = self.candidates.get(self.selected_index).cloned() {
            self.typed = completed;
            true
        } else {
            false
        }
    }

    /// Update candidates based on typed input, filtering from available options.
    /// Uses substring matching (case-insensitive).
    pub fn update_candidates(&mut self, available: &[String]) {
        let typed_lower = self.typed.to_lowercase();
        self.candidates = available
            .iter()
            .filter(|c| c.to_lowercase().contains(&typed_lower))
            .cloned()
            .collect();
        self.selected_index = 0;
    }

    /// Append a character to typed input and update candidates
    pub fn push_char(&mut self, c: char, available: &[String]) {
        self.original_typed = None; // User is now editing, clear completion state
        self.typed.push(c);
        self.update_candidates(available);
    }

    /// Remove last character from typed input and update candidates.
    /// If in a Tab-completed state, reverts to original typed text instead.
    pub fn pop_char(&mut self, available: &[String]) -> bool {
        // If we're in a completed state, revert to original
        if let Some(original) = self.original_typed.take() {
            self.typed = original;
            self.update_candidates(available);
            return true;
        }
        // Otherwise, normal backspace behavior
        if self.typed.pop().is_some() {
            self.update_candidates(available);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ghost_suffix() {
        let state = AutocompleteState::with_typed("sta", vec!["state".into(), "status".into()]);
        assert_eq!(state.ghost_suffix(), Some("te")); // Shows first match suffix
    }

    #[test]
    fn test_ghost_suffix_no_match() {
        let state = AutocompleteState::with_typed("xyz", vec![]);
        assert_eq!(state.ghost_suffix(), None);
    }

    #[test]
    fn test_accept_completion_cycles() {
        let mut state = AutocompleteState::with_typed(
            "sta",
            vec!["state".into(), "status".into(), "start".into()],
        );

        assert!(state.accept_completion());
        assert_eq!(state.typed, "state");

        assert!(state.accept_completion());
        assert_eq!(state.typed, "status");

        assert!(state.accept_completion());
        assert_eq!(state.typed, "start");

        assert!(state.accept_completion());
        assert_eq!(state.typed, "state"); // Wraps around
    }

    #[test]
    fn test_accept_completion_empty() {
        let mut state = AutocompleteState::with_typed("xyz", vec![]);
        assert!(!state.accept_completion());
        assert_eq!(state.typed, "xyz");
    }

    #[test]
    fn test_backspace_reverts_completion() {
        let available = vec!["state".into(), "status".into()];
        let mut state = AutocompleteState::with_typed("sta", available.clone());
        state.update_candidates(&available);

        // Accept completion
        state.accept_completion();
        assert_eq!(state.typed, "state");

        // Backspace should revert to original
        assert!(state.pop_char(&available));
        assert_eq!(state.typed, "sta");

        // Further backspace removes characters normally
        assert!(state.pop_char(&available));
        assert_eq!(state.typed, "st");
    }

    #[test]
    fn test_push_char_clears_completion_state() {
        let available = vec!["state".into(), "status".into()];
        let mut state = AutocompleteState::with_typed("sta", available.clone());
        state.update_candidates(&available);

        // Accept completion
        state.accept_completion();
        assert_eq!(state.typed, "state");

        // Push a char - clears completion state
        state.push_char('x', &available);
        assert_eq!(state.typed, "statex");

        // Backspace now just removes last char (not reverting)
        state.pop_char(&available);
        assert_eq!(state.typed, "state");
    }

    #[test]
    fn test_update_candidates_case_insensitive() {
        let mut state = AutocompleteState {
            typed: "sta".to_string(),
            ..Default::default()
        };
        state.update_candidates(&["State".into(), "STATUS".into()]);
        assert_eq!(state.candidates, vec!["State", "STATUS"]);
    }

    #[test]
    fn test_update_candidates_substring_matching() {
        let mut state = AutocompleteState {
            typed: "run".to_string(),
            ..Default::default()
        };
        state.update_candidates(&["dag_run_id".into(), "running".into(), "state".into()]);
        // Both "dag_run_id" and "running" contain "run"
        assert_eq!(state.candidates, vec!["dag_run_id", "running"]);
    }
}
