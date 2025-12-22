use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent};

use super::{AutocompleteState, FilterCondition, FilterKind, FilterState, FilterableField};

use ratatui::layout::Position;

/// Helper: Get all field names from filterable fields
fn field_names(fields: &[FilterableField]) -> Vec<String> {
    fields.iter().map(|f| f.name.to_string()).collect()
}

/// Helper: Get value candidates for a field based on its kind and available field values
fn value_candidates_for_field(
    field: &str,
    field_kind: &FilterKind,
    field_values: &HashMap<String, Vec<String>>,
) -> Vec<String> {
    match field_kind {
        FilterKind::Enum(values) => values
            .iter()
            .map(std::string::ToString::to_string)
            .collect(),
        FilterKind::FreeText => field_values.get(field).cloned().unwrap_or_default(),
    }
}

/// Helper: Find the `FilterKind` for a field name
fn find_field_kind(field_name: &str, fields: &[FilterableField]) -> FilterKind {
    fields
        .iter()
        .find(|f| f.name == field_name)
        .map_or(FilterKind::FreeText, |f| f.kind.clone())
}

/// The filter state machine that handles keyboard input and state transitions
#[derive(Clone, Debug, Default)]
pub struct FilterStateMachine {
    pub state: FilterState,
    /// Whether space was just pressed (for detecting `:` after space to chain filters)
    space_just_pressed: bool,
    /// Conditions stored when filter is deactivated, restored on reactivation
    stored_conditions: Vec<FilterCondition>,
    /// Cursor position, updated after each render
    pub cursor_position: Position,
    /// Name of the primary field (e.g., "`dag_id`", "name")
    primary_field: Option<String>,
    /// Available values for primary field autocomplete (populated from actual data)
    primary_values: Vec<String>,
    /// Available values for each non-primary field (for `FreeText` field autocomplete)
    field_values: HashMap<String, Vec<String>>,
}

impl FilterStateMachine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the filter is currently active (visible)
    pub const fn is_active(&self) -> bool {
        self.state.is_active()
    }

    /// Get all active filter conditions for live filtering
    /// Returns stored conditions when inactive, active conditions when active
    pub fn active_conditions(&self) -> Vec<FilterCondition> {
        if matches!(self.state, FilterState::Inactive) {
            self.stored_conditions.clone()
        } else {
            self.state.active_conditions(self.primary_field.as_deref())
        }
    }

    /// Activate the filter (called when user presses '/')
    /// Restores previously stored conditions if any
    pub fn activate(&mut self) {
        // Separate primary condition from other conditions
        let (primary, others): (Vec<_>, Vec<_>) =
            self.stored_conditions.iter().cloned().partition(|c| {
                // Match by field name or legacy is_primary flag
                self.primary_field
                    .as_ref()
                    .map_or(c.is_primary, |f| &c.field == f)
            });

        // If there are non-primary conditions, go to AttributeSelection
        // (primary is only suggested when there are no other conditions)
        if others.is_empty() {
            // No non-primary conditions - go to Default mode (suggest primary)
            let mut autocomplete = AutocompleteState::default();

            // If there was a primary filter, restore it to the autocomplete typed field
            if let Some(primary_condition) = primary.into_iter().next() {
                autocomplete.typed = primary_condition.value;
            }

            // Populate candidates from primary values
            autocomplete.update_candidates(&self.primary_values);

            self.state = FilterState::Default {
                autocomplete,
                conditions: vec![],
            };
        } else {
            // Include primary condition in the conditions list (treated as regular condition)
            let mut all_conditions = primary;
            all_conditions.extend(others);

            let autocomplete = AutocompleteState::default();
            // Candidates will be set when update() is called with fields
            self.state = FilterState::AttributeSelection {
                autocomplete,
                conditions: all_conditions,
            };
        }
        self.space_just_pressed = false;
    }

    /// Deactivate the filter, storing current conditions for later restoration
    pub fn deactivate(&mut self) {
        // Store all active conditions (including in-progress typing) before going inactive
        self.stored_conditions = self.state.active_conditions(self.primary_field.as_deref());
        self.state = FilterState::Inactive;
        self.space_just_pressed = false;
    }

    /// Clear all stored conditions (used when user wants to reset filter completely)
    pub fn clear(&mut self) {
        self.stored_conditions.clear();
        self.state = FilterState::Inactive;
        self.space_just_pressed = false;
    }

    /// Get the stored conditions (for filtering even when inactive)
    pub fn stored_conditions(&self) -> &[FilterCondition] {
        &self.stored_conditions
    }

    /// Update the primary field name and available values for autocomplete.
    /// Call this when data is loaded/updated to enable autocomplete suggestions.
    pub fn set_primary_values(&mut self, field_name: &str, values: Vec<String>) {
        self.primary_field = Some(field_name.to_string());
        self.primary_values = values;
    }

    /// Get the primary field name
    pub fn primary_field(&self) -> Option<&str> {
        self.primary_field.as_deref()
    }

    /// Update the available values for a specific field's autocomplete
    /// Call this when data is loaded/updated to enable autocomplete for `FreeText` fields
    pub fn set_field_values(&mut self, field: &str, values: Vec<String>) {
        self.field_values.insert(field.to_string(), values);
    }

    /// Get a display string for the active filter (for showing in title bar)
    /// Returns None if no active conditions
    pub fn filter_display(&self) -> Option<String> {
        let conditions = self.active_conditions();
        if conditions.is_empty() {
            return None;
        }

        Some(
            conditions
                .iter()
                .map(|c| format!("{}:{}", c.field, c.value))
                .collect::<Vec<_>>()
                .join(" "),
        )
    }

    /// Helper: Create `ValueInput` state for editing an existing condition
    fn edit_condition_state(
        condition: FilterCondition,
        remaining_conditions: Vec<FilterCondition>,
        fields: &[FilterableField],
        field_values: &HashMap<String, Vec<String>>,
    ) -> FilterState {
        let kind = find_field_kind(&condition.field, fields);
        let candidates = value_candidates_for_field(&condition.field, &kind, field_values);
        FilterState::ValueInput {
            field: condition.field,
            field_kind: kind,
            autocomplete: AutocompleteState::with_typed_and_candidates(
                condition.value,
                &candidates,
            ),
            conditions: remaining_conditions,
        }
    }

    /// Helper: Create new conditions list by adding current typed value if non-empty
    fn confirm_current_value(
        conditions: &[FilterCondition],
        field: &str,
        typed: &str,
    ) -> Vec<FilterCondition> {
        let mut new_conditions = conditions.to_vec();
        if !typed.is_empty() {
            new_conditions.push(FilterCondition::new(field, typed, false));
        }
        new_conditions
    }

    /// Handle a key event. Returns true if the event was consumed.
    pub fn update(&mut self, key: &KeyEvent, fields: &[FilterableField]) -> bool {
        let primary_values = self.primary_values.clone();
        let field_values = self.field_values.clone();
        let all_field_names = field_names(fields);

        match &mut self.state {
            FilterState::Inactive => {
                if key.code == KeyCode::Char('/') {
                    self.activate();
                    return true;
                }
                false
            }

            FilterState::Default {
                autocomplete,
                conditions,
            } => {
                self.space_just_pressed = false;

                match key.code {
                    KeyCode::Esc | KeyCode::Enter => {
                        self.deactivate();
                        true
                    }
                    KeyCode::Char(':') => {
                        self.state = FilterState::AttributeSelection {
                            autocomplete: AutocompleteState::with_typed_and_candidates(
                                "",
                                &all_field_names,
                            ),
                            conditions: conditions.clone(),
                        };
                        true
                    }
                    KeyCode::Backspace => {
                        if !autocomplete.pop_char(&primary_values) {
                            self.state = if let Some(last) = conditions.pop() {
                                Self::edit_condition_state(
                                    last,
                                    conditions.clone(),
                                    fields,
                                    &field_values,
                                )
                            } else {
                                FilterState::AttributeSelection {
                                    autocomplete: AutocompleteState::with_typed_and_candidates(
                                        "",
                                        &all_field_names,
                                    ),
                                    conditions: vec![],
                                }
                            };
                        }
                        true
                    }
                    KeyCode::Tab => {
                        autocomplete.accept_completion();
                        true
                    }
                    KeyCode::BackTab => {
                        autocomplete.accept_prev_completion();
                        true
                    }
                    KeyCode::Char(c) => {
                        autocomplete.push_char(c, &primary_values);
                        true
                    }
                    _ => false,
                }
            }

            FilterState::AttributeSelection {
                autocomplete,
                conditions,
            } => {
                self.space_just_pressed = false;

                match key.code {
                    KeyCode::Esc | KeyCode::Enter => {
                        self.deactivate();
                        true
                    }
                    KeyCode::Char(' ') => {
                        // Confirm attribute: use typed if valid, else first candidate
                        let selected = if autocomplete.candidates.contains(&autocomplete.typed) {
                            Some(autocomplete.typed.clone())
                        } else {
                            autocomplete.candidates.first().cloned()
                        };

                        if let Some(selected) = selected {
                            let kind = find_field_kind(&selected, fields);
                            let candidates =
                                value_candidates_for_field(&selected, &kind, &field_values);
                            self.state = FilterState::ValueInput {
                                field: selected,
                                field_kind: kind,
                                autocomplete: AutocompleteState::with_typed_and_candidates(
                                    "",
                                    &candidates,
                                ),
                                conditions: conditions.clone(),
                            };
                        }
                        true
                    }
                    KeyCode::Backspace => {
                        if !autocomplete.pop_char(&all_field_names) {
                            self.state = if let Some(last) = conditions.pop() {
                                Self::edit_condition_state(
                                    last,
                                    conditions.clone(),
                                    fields,
                                    &field_values,
                                )
                            } else {
                                FilterState::Default {
                                    autocomplete: AutocompleteState::with_typed_and_candidates(
                                        "",
                                        &primary_values,
                                    ),
                                    conditions: vec![],
                                }
                            };
                        }
                        true
                    }
                    KeyCode::Tab => {
                        autocomplete.accept_completion();
                        true
                    }
                    KeyCode::BackTab => {
                        autocomplete.accept_prev_completion();
                        true
                    }
                    KeyCode::Char(c) => {
                        autocomplete.push_char(c, &all_field_names);
                        true
                    }
                    _ => false,
                }
            }

            FilterState::ValueInput {
                field,
                field_kind,
                autocomplete,
                conditions,
            } => {
                let value_candidates = value_candidates_for_field(field, field_kind, &field_values);

                match key.code {
                    KeyCode::Esc | KeyCode::Enter => {
                        self.deactivate();
                        true
                    }
                    KeyCode::Char(':') if self.space_just_pressed => {
                        let new_conditions =
                            Self::confirm_current_value(conditions, field, &autocomplete.typed);
                        self.state = FilterState::AttributeSelection {
                            autocomplete: AutocompleteState::with_typed_and_candidates(
                                "",
                                &all_field_names,
                            ),
                            conditions: new_conditions,
                        };
                        self.space_just_pressed = false;
                        true
                    }
                    KeyCode::Char(' ') => {
                        let new_conditions =
                            Self::confirm_current_value(conditions, field, &autocomplete.typed);

                        // Go to AttributeSelection if conditions exist, else Default
                        self.state = if new_conditions.is_empty() {
                            FilterState::Default {
                                autocomplete: AutocompleteState::with_typed_and_candidates(
                                    "",
                                    &primary_values,
                                ),
                                conditions: vec![],
                            }
                        } else {
                            FilterState::AttributeSelection {
                                autocomplete: AutocompleteState::with_typed_and_candidates(
                                    "",
                                    &all_field_names,
                                ),
                                conditions: new_conditions,
                            }
                        };
                        self.space_just_pressed = true;
                        true
                    }
                    KeyCode::Backspace => {
                        self.space_just_pressed = false;
                        if !autocomplete.pop_char(&value_candidates) {
                            let mut new_autocomplete =
                                AutocompleteState::with_typed_and_candidates("", &all_field_names);
                            new_autocomplete.typed.clone_from(field);
                            self.state = FilterState::AttributeSelection {
                                autocomplete: new_autocomplete,
                                conditions: conditions.clone(),
                            };
                        }
                        true
                    }
                    KeyCode::Tab => {
                        self.space_just_pressed = false;
                        autocomplete.accept_completion();
                        true
                    }
                    KeyCode::BackTab => {
                        self.space_just_pressed = false;
                        autocomplete.accept_prev_completion();
                        true
                    }
                    KeyCode::Char(c) => {
                        self.space_just_pressed = false;
                        autocomplete.push_char(c, &value_candidates);
                        true
                    }
                    _ => {
                        self.space_just_pressed = false;
                        false
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEvent, KeyModifiers};

    /// Test harness for `FilterStateMachine` tests
    struct TestHarness {
        fsm: FilterStateMachine,
        fields: Vec<FilterableField>,
    }

    impl TestHarness {
        fn new() -> Self {
            Self {
                fsm: FilterStateMachine::new(),
                fields: vec![
                    FilterableField::primary("dag_run_id"),
                    FilterableField::enumerated("state", vec!["running", "success", "failed"]),
                    FilterableField::enumerated("run_type", vec!["scheduled", "manual"]),
                ],
            }
        }

        /// Send a sequence of keys, where each element can be a char or special key
        fn send(&mut self, keys: &[Key]) {
            for k in keys {
                self.fsm.update(&k.into(), &self.fields);
            }
        }

        /// Type a string (each char becomes a Char key event)
        fn type_str(&mut self, s: &str) {
            for c in s.chars() {
                self.fsm.update(&key(KeyCode::Char(c)), &self.fields);
            }
        }

        /// Build filter `:field value ` and confirm it
        fn add_filter(&mut self, field: &str, value: &str) {
            self.send(&[Key::Colon]);
            self.type_str(field);
            self.send(&[Key::Space]);
            self.type_str(value);
            self.send(&[Key::Space]);
        }

        fn activate(&mut self) {
            self.fsm.activate();
        }

        fn assert_state_default(&self) {
            assert!(matches!(self.fsm.state, FilterState::Default { .. }));
        }

        fn assert_state_attribute_selection(&self) {
            assert!(matches!(
                self.fsm.state,
                FilterState::AttributeSelection { .. }
            ));
        }

        fn assert_state_value_input(&self) {
            assert!(matches!(self.fsm.state, FilterState::ValueInput { .. }));
        }

        fn assert_typed(&self, expected: &str) {
            let typed = self.fsm.state.autocomplete().map(|a| a.typed.as_str());
            assert_eq!(typed, Some(expected));
        }

        fn assert_candidates(&self, expected: &[&str]) {
            let candidates: Vec<&str> = self
                .fsm
                .state
                .autocomplete()
                .map(|a| a.candidates.iter().map(String::as_str).collect())
                .unwrap_or_default();
            assert_eq!(candidates, expected);
        }

        fn assert_conditions_count(&self, count: usize) {
            assert_eq!(self.fsm.active_conditions().len(), count);
        }

        fn assert_condition(&self, index: usize, field: &str, value: &str) {
            let conditions = self.fsm.active_conditions();
            assert_eq!(conditions[index].field, field);
            assert_eq!(conditions[index].value, value);
        }

        fn assert_primary_condition(&self, index: usize, value: &str) {
            let conditions = self.fsm.active_conditions();
            assert!(conditions[index].is_primary);
            assert_eq!(conditions[index].value, value);
        }
    }

    /// Key variants for cleaner test syntax
    #[derive(Clone, Copy)]
    enum Key {
        Colon,
        Space,
        Tab,
        Backspace,
        Esc,
        Slash,
    }

    impl From<&Key> for KeyEvent {
        fn from(k: &Key) -> Self {
            key(match k {
                Key::Colon => KeyCode::Char(':'),
                Key::Space => KeyCode::Char(' '),
                Key::Tab => KeyCode::Tab,
                Key::Backspace => KeyCode::Backspace,
                Key::Esc => KeyCode::Esc,
                Key::Slash => KeyCode::Char('/'),
            })
        }
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn test_activate_on_slash() {
        let mut h = TestHarness::new();

        assert!(!h.fsm.is_active());
        h.send(&[Key::Slash]);
        assert!(h.fsm.is_active());
    }

    #[test]
    fn test_deactivate_on_esc() {
        let mut h = TestHarness::new();

        h.activate();
        h.send(&[Key::Esc]);
        assert!(!h.fsm.is_active());
    }

    #[test]
    fn test_type_in_default_mode() {
        let mut h = TestHarness::new();

        h.activate();
        h.type_str("my");

        h.assert_conditions_count(1);
        h.assert_primary_condition(0, "my");
    }

    #[test]
    fn test_transition_to_attribute_selection() {
        let mut h = TestHarness::new();

        h.activate();
        h.send(&[Key::Colon]);
        h.assert_state_attribute_selection();
    }

    #[test]
    fn test_attribute_autocomplete() {
        let mut h = TestHarness::new();

        h.activate();
        h.send(&[Key::Colon]);
        h.type_str("s");

        h.assert_typed("s");
        h.assert_candidates(&["state"]);
    }

    #[test]
    fn test_confirm_attribute_and_enter_value() {
        let mut h = TestHarness::new();

        h.activate();
        h.send(&[Key::Colon]);
        h.type_str("sta");
        h.send(&[Key::Space]);

        h.assert_state_value_input();
        if let FilterState::ValueInput {
            field,
            autocomplete,
            ..
        } = &h.fsm.state
        {
            assert_eq!(field, "state");
            assert!(autocomplete.candidates.contains(&"running".to_string()));
        }
    }

    #[test]
    fn test_value_input_with_enum() {
        let mut h = TestHarness::new();

        h.activate();
        h.send(&[Key::Colon]);
        h.type_str("sta");
        h.send(&[Key::Space]);
        h.type_str("r");

        h.assert_typed("r");
        h.assert_candidates(&["running"]);
        h.assert_condition(0, "state", "r");
    }

    #[test]
    fn test_chain_filters() {
        let mut h = TestHarness::new();

        h.activate();
        h.add_filter("sta", "run");

        // After confirming, should be in AttributeSelection (primary only suggested when no conditions)
        h.assert_state_attribute_selection();
        assert_eq!(h.fsm.state.confirmed_conditions().len(), 1);

        // Add another filter
        h.type_str("r");
        h.send(&[Key::Tab, Key::Space]); // Complete to "run_type" and confirm

        h.assert_state_value_input();
        assert_eq!(h.fsm.state.confirmed_conditions().len(), 1);
    }

    #[test]
    fn test_backspace_transitions() {
        let mut h = TestHarness::new();

        h.activate();
        h.send(&[Key::Colon]);
        h.type_str("s");
        h.send(&[Key::Space]);
        h.type_str("r");

        h.assert_state_value_input();

        // Backspace removes 'r', still in ValueInput
        h.send(&[Key::Backspace]);
        h.assert_state_value_input();

        // Backspace again goes to AttributeSelection
        h.send(&[Key::Backspace]);
        h.assert_state_attribute_selection();
    }

    #[test]
    fn test_backspace_removes_primary_field() {
        let mut h = TestHarness::new();

        h.activate();
        h.send(&[Key::Backspace]);

        h.assert_state_attribute_selection();
        h.assert_candidates(&["dag_run_id", "state", "run_type"]);
    }

    #[test]
    fn test_tab_cycles_through_completions() {
        let mut h = TestHarness::new();

        h.activate();
        h.send(&[Key::Colon]);

        h.assert_candidates(&["dag_run_id", "state", "run_type"]);

        // Tab cycles through candidates
        let expected = ["dag_run_id", "state", "run_type", "dag_run_id"];
        for exp in expected {
            h.send(&[Key::Tab]);
            h.assert_typed(exp);
        }
    }

    #[test]
    fn test_filter_persistence_on_deactivate_reactivate() {
        let mut h = TestHarness::new();

        h.activate();
        h.add_filter("sta", "run");

        assert_eq!(h.fsm.state.confirmed_conditions().len(), 1);

        h.send(&[Key::Esc]);
        assert!(!h.fsm.is_active());

        // Conditions persist while inactive
        h.assert_conditions_count(1);
        h.assert_condition(0, "state", "run");

        // Reactivate restores conditions
        h.activate();
        assert!(h.fsm.is_active());
        assert_eq!(h.fsm.state.confirmed_conditions().len(), 1);
    }

    #[test]
    fn test_primary_filter_persistence() {
        let mut h = TestHarness::new();

        h.activate();
        h.type_str("my_d");

        h.assert_conditions_count(1);
        h.assert_primary_condition(0, "my_d");

        h.send(&[Key::Esc]);
        h.assert_primary_condition(0, "my_d");

        // Reactivate - typed text restored in Default mode
        h.activate();
        h.assert_state_default();
        h.assert_typed("my_d");

        // Continue typing
        h.type_str("ag");
        h.assert_primary_condition(0, "my_dag");
    }

    #[test]
    fn test_clear_removes_stored_conditions() {
        let mut h = TestHarness::new();

        h.activate();
        h.add_filter("s", "r");
        h.send(&[Key::Esc]);

        h.fsm.clear();
        assert!(h.fsm.active_conditions().is_empty());
        assert!(h.fsm.stored_conditions().is_empty());
    }

    #[test]
    fn test_backspace_edits_committed_filter() {
        let mut h = TestHarness::new();

        h.activate();
        h.add_filter("sta", "run");

        h.assert_state_attribute_selection();
        assert_eq!(h.fsm.state.confirmed_conditions().len(), 1);

        // Backspace when empty goes to ValueInput with last condition for editing
        h.send(&[Key::Backspace]);

        h.assert_state_value_input();
        if let FilterState::ValueInput {
            field,
            autocomplete,
            conditions,
            ..
        } = &h.fsm.state
        {
            assert_eq!(field, "state");
            assert_eq!(autocomplete.typed, "run");
            assert!(conditions.is_empty());
        }

        // Backspace removes characters from value
        h.send(&[Key::Backspace, Key::Backspace, Key::Backspace]);
        h.assert_typed("");

        // One more goes to AttributeSelection
        h.send(&[Key::Backspace]);
        h.assert_state_attribute_selection();
    }
}
