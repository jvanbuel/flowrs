# Filter State Machine Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the simple substring filter with a state machine supporting autocomplete, multi-attribute filtering, and trait-based field discovery.

**Architecture:** The filter becomes a generic `FilterStateMachine<T>` where `T: Filterable`. The `Filterable` trait (implemented manually for each filterable type) exposes which fields can be filtered and their value types (free-text vs enum). The state machine handles keyboard input, maintains autocomplete state, and provides conditions for live filtering.

**Tech Stack:** Rust, ratatui (TUI), crossterm (keyboard input)

---

## Task 1: Create AutocompleteState Module

**Files:**
- Create: `src/app/model/filter/autocomplete.rs`
- Create: `src/app/model/filter/mod.rs`
- Modify: `src/app/model.rs:5-11`

**Step 1: Write the failing test**

Create `src/app/model/filter/autocomplete.rs`:

```rust
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AutocompleteState {
    pub typed: String,
    pub candidates: Vec<String>,
    pub selected_index: usize,
}

impl AutocompleteState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the suffix to display as ghost text (grayed out)
    pub fn ghost_suffix(&self) -> Option<&str> {
        self.candidates
            .get(self.selected_index)
            .and_then(|c| c.strip_prefix(&self.typed))
    }

    /// Returns the currently selected candidate
    pub fn selected(&self) -> Option<&str> {
        self.candidates.get(self.selected_index).map(|s| s.as_str())
    }

    /// Cycle to the next candidate (wrapping)
    pub fn cycle_next(&mut self) {
        if !self.candidates.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.candidates.len();
        }
    }

    /// Cycle to the previous candidate (wrapping)
    pub fn cycle_prev(&mut self) {
        if !self.candidates.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.candidates.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    /// Update candidates based on typed input, filtering from available options
    pub fn update_candidates(&mut self, available: &[String]) {
        let typed_lower = self.typed.to_lowercase();
        self.candidates = available
            .iter()
            .filter(|c| c.to_lowercase().starts_with(&typed_lower))
            .cloned()
            .collect();
        self.selected_index = 0;
    }

    /// Append a character to typed input and update candidates
    pub fn push_char(&mut self, c: char, available: &[String]) {
        self.typed.push(c);
        self.update_candidates(available);
    }

    /// Remove last character from typed input and update candidates
    pub fn pop_char(&mut self, available: &[String]) -> bool {
        if self.typed.pop().is_some() {
            self.update_candidates(available);
            true
        } else {
            false
        }
    }

    /// Clear all state
    pub fn clear(&mut self) {
        self.typed.clear();
        self.candidates.clear();
        self.selected_index = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ghost_suffix_with_match() {
        let state = AutocompleteState {
            typed: "sta".to_string(),
            candidates: vec!["state".to_string(), "status".to_string()],
            selected_index: 0,
        };
        assert_eq!(state.ghost_suffix(), Some("te"));
    }

    #[test]
    fn test_ghost_suffix_second_candidate() {
        let state = AutocompleteState {
            typed: "sta".to_string(),
            candidates: vec!["state".to_string(), "status".to_string()],
            selected_index: 1,
        };
        assert_eq!(state.ghost_suffix(), Some("tus"));
    }

    #[test]
    fn test_ghost_suffix_no_candidates() {
        let state = AutocompleteState {
            typed: "xyz".to_string(),
            candidates: vec![],
            selected_index: 0,
        };
        assert_eq!(state.ghost_suffix(), None);
    }

    #[test]
    fn test_cycle_next() {
        let mut state = AutocompleteState {
            typed: "s".to_string(),
            candidates: vec!["state".to_string(), "status".to_string(), "success".to_string()],
            selected_index: 0,
        };

        state.cycle_next();
        assert_eq!(state.selected_index, 1);

        state.cycle_next();
        assert_eq!(state.selected_index, 2);

        state.cycle_next();
        assert_eq!(state.selected_index, 0); // Wraps
    }

    #[test]
    fn test_cycle_prev() {
        let mut state = AutocompleteState {
            typed: "s".to_string(),
            candidates: vec!["state".to_string(), "status".to_string()],
            selected_index: 0,
        };

        state.cycle_prev();
        assert_eq!(state.selected_index, 1); // Wraps to end

        state.cycle_prev();
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_cycle_empty_candidates() {
        let mut state = AutocompleteState::default();
        state.cycle_next();
        assert_eq!(state.selected_index, 0);
        state.cycle_prev();
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_update_candidates() {
        let mut state = AutocompleteState::default();
        let available = vec![
            "state".to_string(),
            "status".to_string(),
            "running".to_string(),
        ];

        state.typed = "st".to_string();
        state.update_candidates(&available);

        assert_eq!(state.candidates, vec!["state", "status"]);
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_update_candidates_case_insensitive() {
        let mut state = AutocompleteState::default();
        let available = vec!["State".to_string(), "STATUS".to_string()];

        state.typed = "sta".to_string();
        state.update_candidates(&available);

        assert_eq!(state.candidates, vec!["State", "STATUS"]);
    }

    #[test]
    fn test_push_char() {
        let mut state = AutocompleteState::default();
        let available = vec!["state".to_string(), "status".to_string()];

        state.push_char('s', &available);
        assert_eq!(state.typed, "s");
        assert_eq!(state.candidates.len(), 2);

        state.push_char('t', &available);
        assert_eq!(state.typed, "st");
        assert_eq!(state.candidates.len(), 2);

        state.push_char('a', &available);
        assert_eq!(state.typed, "sta");
        assert_eq!(state.candidates, vec!["state", "status"]);
    }

    #[test]
    fn test_pop_char() {
        let mut state = AutocompleteState {
            typed: "sta".to_string(),
            candidates: vec!["state".to_string()],
            selected_index: 0,
        };
        let available = vec!["state".to_string(), "status".to_string(), "running".to_string()];

        assert!(state.pop_char(&available));
        assert_eq!(state.typed, "st");
        assert_eq!(state.candidates, vec!["state", "status"]);

        // Pop on empty returns false
        state.typed.clear();
        assert!(!state.pop_char(&available));
    }
}
```

**Step 2: Create the mod.rs file**

Create `src/app/model/filter/mod.rs`:

```rust
mod autocomplete;

pub use autocomplete::AutocompleteState;

// Re-export the old Filter for now (will be replaced later)
mod legacy;
pub use legacy::{CursorState, Filter};
```

**Step 3: Move existing filter.rs to legacy.rs**

Rename `src/app/model/filter.rs` to `src/app/model/filter/legacy.rs`.

**Step 4: Update module declaration in model.rs**

In `src/app/model.rs`, the `pub mod filter;` already exists, but now it points to a directory. Rust handles this automatically.

**Step 5: Run tests to verify**

Run: `cargo test autocomplete`
Expected: All tests pass

**Step 6: Commit**

```bash
git add src/app/model/filter/
git commit -m "feat(filter): add AutocompleteState with ghost text support"
```

---

## Task 2: Create FilterCondition Module

**Files:**
- Create: `src/app/model/filter/condition.rs`
- Modify: `src/app/model/filter/mod.rs`

**Step 1: Write the condition struct with tests**

Create `src/app/model/filter/condition.rs`:

```rust
/// A single filter condition (e.g., "state contains 'running'")
#[derive(Clone, Debug, PartialEq)]
pub struct FilterCondition {
    /// The field name to filter on (e.g., "state", "dag_run_id")
    pub field: String,
    /// The value to match (substring match)
    pub value: String,
    /// Whether this is filtering on the primary/default field
    pub is_primary: bool,
}

impl FilterCondition {
    pub fn new(field: impl Into<String>, value: impl Into<String>, is_primary: bool) -> Self {
        Self {
            field: field.into(),
            value: value.into(),
            is_primary,
        }
    }

    pub fn primary(value: impl Into<String>) -> Self {
        Self {
            field: String::new(),
            value: value.into(),
            is_primary: true,
        }
    }

    /// Check if this condition matches a field value (case-insensitive substring)
    pub fn matches(&self, field_value: &str) -> bool {
        field_value
            .to_lowercase()
            .contains(&self.value.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_condition_matches() {
        let cond = FilterCondition::new("state", "run", false);

        assert!(cond.matches("running"));
        assert!(cond.matches("RUNNING"));
        assert!(cond.matches("up_for_running"));
        assert!(!cond.matches("success"));
    }

    #[test]
    fn test_condition_matches_case_insensitive() {
        let cond = FilterCondition::new("state", "RUN", false);

        assert!(cond.matches("running"));
        assert!(cond.matches("RUNNING"));
    }

    #[test]
    fn test_primary_condition() {
        let cond = FilterCondition::primary("my_dag");

        assert!(cond.is_primary);
        assert!(cond.matches("my_dag_v2"));
    }
}
```

**Step 2: Update mod.rs**

Add to `src/app/model/filter/mod.rs`:

```rust
mod condition;
pub use condition::FilterCondition;
```

**Step 3: Run tests**

Run: `cargo test condition`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/app/model/filter/condition.rs src/app/model/filter/mod.rs
git commit -m "feat(filter): add FilterCondition for individual filter criteria"
```

---

## Task 3: Create Filterable Trait and FilterKind

**Files:**
- Create: `src/app/model/filter/filterable.rs`
- Modify: `src/app/model/filter/mod.rs`

**Step 1: Write the filterable trait**

Create `src/app/model/filter/filterable.rs`:

```rust
/// Describes the kind of filter values a field accepts
#[derive(Clone, Debug, PartialEq)]
pub enum FilterKind {
    /// Free-text field with no predefined values
    FreeText,
    /// Field with known enum-like values for autocomplete
    Enum(Vec<&'static str>),
}

impl FilterKind {
    /// Get the available values for autocomplete (empty for FreeText)
    pub fn values(&self) -> Vec<String> {
        match self {
            FilterKind::FreeText => vec![],
            FilterKind::Enum(values) => values.iter().map(|s| (*s).to_string()).collect(),
        }
    }
}

/// Describes a field that can be filtered
#[derive(Clone, Debug)]
pub struct FilterableField {
    /// The field name as it appears in the struct
    pub name: &'static str,
    /// What kind of values this field accepts
    pub kind: FilterKind,
    /// Whether this is the primary/default filter field
    pub is_primary: bool,
}

impl FilterableField {
    pub fn primary(name: &'static str) -> Self {
        Self {
            name,
            kind: FilterKind::FreeText,
            is_primary: true,
        }
    }

    pub fn free_text(name: &'static str) -> Self {
        Self {
            name,
            kind: FilterKind::FreeText,
            is_primary: false,
        }
    }

    pub fn enumerated(name: &'static str, values: Vec<&'static str>) -> Self {
        Self {
            name,
            kind: FilterKind::Enum(values),
            is_primary: false,
        }
    }
}

/// Trait for types that can be filtered in the TUI
pub trait Filterable {
    /// Returns the name of the primary filter field
    fn primary_field() -> &'static str;

    /// Returns all filterable fields with their metadata
    fn filterable_fields() -> Vec<FilterableField>;

    /// Get the value of a field by name for filtering
    fn get_field_value(&self, field_name: &str) -> Option<String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_kind_values() {
        let free = FilterKind::FreeText;
        assert!(free.values().is_empty());

        let enumerated = FilterKind::Enum(vec!["running", "success", "failed"]);
        assert_eq!(enumerated.values(), vec!["running", "success", "failed"]);
    }

    #[test]
    fn test_filterable_field_constructors() {
        let primary = FilterableField::primary("dag_id");
        assert!(primary.is_primary);
        assert_eq!(primary.name, "dag_id");

        let free = FilterableField::free_text("description");
        assert!(!free.is_primary);
        assert!(matches!(free.kind, FilterKind::FreeText));

        let enumerated = FilterableField::enumerated("state", vec!["running", "success"]);
        assert!(!enumerated.is_primary);
        assert!(matches!(enumerated.kind, FilterKind::Enum(_)));
    }
}
```

**Step 2: Update mod.rs**

Add to `src/app/model/filter/mod.rs`:

```rust
mod filterable;
pub use filterable::{FilterKind, FilterableField, Filterable};
```

**Step 3: Run tests**

Run: `cargo test filterable`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/app/model/filter/filterable.rs src/app/model/filter/mod.rs
git commit -m "feat(filter): add Filterable trait and FilterKind for reflection"
```

---

## Task 4: Create FilterState Enum

**Files:**
- Create: `src/app/model/filter/state.rs`
- Modify: `src/app/model/filter/mod.rs`

**Step 1: Write the state enum**

Create `src/app/model/filter/state.rs`:

```rust
use super::{AutocompleteState, FilterCondition, FilterKind};

/// The filter state machine states
#[derive(Clone, Debug)]
pub enum FilterState {
    /// Filter UI is hidden
    Inactive,

    /// Filtering on the primary field (default mode after pressing '/')
    Default {
        autocomplete: AutocompleteState,
        conditions: Vec<FilterCondition>,
    },

    /// Selecting which attribute to filter on (after pressing ':')
    AttributeSelection {
        autocomplete: AutocompleteState,
        conditions: Vec<FilterCondition>,
    },

    /// Entering a value for the selected attribute
    ValueInput {
        field: String,
        field_kind: FilterKind,
        autocomplete: AutocompleteState,
        conditions: Vec<FilterCondition>,
    },
}

impl Default for FilterState {
    fn default() -> Self {
        Self::Inactive
    }
}

impl FilterState {
    /// Check if the filter is currently active (visible)
    pub fn is_active(&self) -> bool {
        !matches!(self, Self::Inactive)
    }

    /// Get all currently active filter conditions (including in-progress typing)
    pub fn active_conditions(&self) -> Vec<FilterCondition> {
        match self {
            Self::Inactive => vec![],

            Self::Default { autocomplete, conditions } => {
                let mut result = conditions.clone();
                if !autocomplete.typed.is_empty() {
                    result.push(FilterCondition::primary(&autocomplete.typed));
                }
                result
            }

            Self::AttributeSelection { conditions, .. } => {
                // While selecting attribute, only apply confirmed conditions
                conditions.clone()
            }

            Self::ValueInput { field, autocomplete, conditions, .. } => {
                let mut result = conditions.clone();
                if !autocomplete.typed.is_empty() {
                    result.push(FilterCondition::new(field, &autocomplete.typed, false));
                }
                result
            }
        }
    }

    /// Get confirmed conditions only (not including in-progress typing)
    pub fn confirmed_conditions(&self) -> &[FilterCondition] {
        match self {
            Self::Inactive => &[],
            Self::Default { conditions, .. }
            | Self::AttributeSelection { conditions, .. }
            | Self::ValueInput { conditions, .. } => conditions,
        }
    }

    /// Get the current autocomplete state if any
    pub fn autocomplete(&self) -> Option<&AutocompleteState> {
        match self {
            Self::Inactive => None,
            Self::Default { autocomplete, .. }
            | Self::AttributeSelection { autocomplete, .. }
            | Self::ValueInput { autocomplete, .. } => Some(autocomplete),
        }
    }

    /// Get mutable reference to autocomplete state
    pub fn autocomplete_mut(&mut self) -> Option<&mut AutocompleteState> {
        match self {
            Self::Inactive => None,
            Self::Default { autocomplete, .. }
            | Self::AttributeSelection { autocomplete, .. }
            | Self::ValueInput { autocomplete, .. } => Some(autocomplete),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_active() {
        assert!(!FilterState::Inactive.is_active());

        let default = FilterState::Default {
            autocomplete: AutocompleteState::default(),
            conditions: vec![],
        };
        assert!(default.is_active());
    }

    #[test]
    fn test_active_conditions_default_with_typing() {
        let state = FilterState::Default {
            autocomplete: AutocompleteState {
                typed: "my_dag".to_string(),
                candidates: vec![],
                selected_index: 0,
            },
            conditions: vec![],
        };

        let conditions = state.active_conditions();
        assert_eq!(conditions.len(), 1);
        assert!(conditions[0].is_primary);
        assert_eq!(conditions[0].value, "my_dag");
    }

    #[test]
    fn test_active_conditions_with_confirmed_and_typing() {
        let confirmed = FilterCondition::new("state", "running", false);
        let state = FilterState::ValueInput {
            field: "dag_id".to_string(),
            field_kind: FilterKind::FreeText,
            autocomplete: AutocompleteState {
                typed: "my".to_string(),
                candidates: vec![],
                selected_index: 0,
            },
            conditions: vec![confirmed.clone()],
        };

        let conditions = state.active_conditions();
        assert_eq!(conditions.len(), 2);
        assert_eq!(conditions[0], confirmed);
        assert_eq!(conditions[1].field, "dag_id");
        assert_eq!(conditions[1].value, "my");
    }

    #[test]
    fn test_attribute_selection_only_confirmed() {
        let confirmed = FilterCondition::new("state", "running", false);
        let state = FilterState::AttributeSelection {
            autocomplete: AutocompleteState {
                typed: "dag".to_string(), // This is attribute name being typed, not a filter
                candidates: vec!["dag_id".to_string()],
                selected_index: 0,
            },
            conditions: vec![confirmed.clone()],
        };

        let conditions = state.active_conditions();
        assert_eq!(conditions.len(), 1);
        assert_eq!(conditions[0], confirmed);
    }
}
```

**Step 2: Update mod.rs**

Add to `src/app/model/filter/mod.rs`:

```rust
mod state;
pub use state::FilterState;
```

**Step 3: Run tests**

Run: `cargo test state`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/app/model/filter/state.rs src/app/model/filter/mod.rs
git commit -m "feat(filter): add FilterState enum for state machine"
```

---

## Task 5: Create FilterStateMachine with Keyboard Handling

**Files:**
- Create: `src/app/model/filter/state_machine.rs`
- Modify: `src/app/model/filter/mod.rs`

**Step 1: Write the state machine**

Create `src/app/model/filter/state_machine.rs`:

```rust
use crossterm::event::{KeyCode, KeyEvent};

use super::{
    AutocompleteState, FilterCondition, FilterKind, FilterState, FilterableField,
};

/// The filter state machine that handles keyboard input and state transitions
#[derive(Clone, Debug, Default)]
pub struct FilterStateMachine {
    pub state: FilterState,
    /// Whether space was just pressed (for detecting `:` after space to chain filters)
    space_just_pressed: bool,
}

impl FilterStateMachine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the filter is currently active (visible)
    pub fn is_active(&self) -> bool {
        self.state.is_active()
    }

    /// Get all active filter conditions for live filtering
    pub fn active_conditions(&self) -> Vec<FilterCondition> {
        self.state.active_conditions()
    }

    /// Activate the filter (called when user presses '/')
    pub fn activate(&mut self) {
        self.state = FilterState::Default {
            autocomplete: AutocompleteState::default(),
            conditions: vec![],
        };
        self.space_just_pressed = false;
    }

    /// Deactivate the filter
    pub fn deactivate(&mut self) {
        self.state = FilterState::Inactive;
        self.space_just_pressed = false;
    }

    /// Handle a key event. Returns true if the event was consumed.
    pub fn update(&mut self, key: &KeyEvent, fields: &[FilterableField]) -> bool {
        match &mut self.state {
            FilterState::Inactive => {
                if key.code == KeyCode::Char('/') {
                    self.activate();
                    return true;
                }
                false
            }

            FilterState::Default { autocomplete, conditions } => {
                self.space_just_pressed = false;
                match key.code {
                    KeyCode::Esc | KeyCode::Enter => {
                        self.deactivate();
                        true
                    }
                    KeyCode::Char(':') => {
                        // Transition to attribute selection
                        let field_names: Vec<String> = fields
                            .iter()
                            .filter(|f| !f.is_primary)
                            .map(|f| f.name.to_string())
                            .collect();

                        let mut new_autocomplete = AutocompleteState::default();
                        new_autocomplete.update_candidates(&field_names);

                        self.state = FilterState::AttributeSelection {
                            autocomplete: new_autocomplete,
                            conditions: conditions.clone(),
                        };
                        true
                    }
                    KeyCode::Backspace => {
                        let available = Self::get_primary_values(fields);
                        autocomplete.pop_char(&available);
                        true
                    }
                    KeyCode::Tab => {
                        autocomplete.cycle_next();
                        true
                    }
                    KeyCode::BackTab => {
                        autocomplete.cycle_prev();
                        true
                    }
                    KeyCode::Char(c) => {
                        let available = Self::get_primary_values(fields);
                        autocomplete.push_char(c, &available);
                        true
                    }
                    _ => false,
                }
            }

            FilterState::AttributeSelection { autocomplete, conditions } => {
                self.space_just_pressed = false;
                let field_names: Vec<String> = fields
                    .iter()
                    .filter(|f| !f.is_primary)
                    .map(|f| f.name.to_string())
                    .collect();

                match key.code {
                    KeyCode::Esc | KeyCode::Enter => {
                        self.deactivate();
                        true
                    }
                    KeyCode::Char(' ') => {
                        // Confirm attribute selection
                        if let Some(selected) = autocomplete.selected().map(String::from) {
                            let field_kind = fields
                                .iter()
                                .find(|f| f.name == selected)
                                .map(|f| f.kind.clone())
                                .unwrap_or(FilterKind::FreeText);

                            let value_candidates = field_kind.values();
                            let mut new_autocomplete = AutocompleteState::default();
                            new_autocomplete.update_candidates(&value_candidates);

                            self.state = FilterState::ValueInput {
                                field: selected,
                                field_kind,
                                autocomplete: new_autocomplete,
                                conditions: conditions.clone(),
                            };
                        }
                        true
                    }
                    KeyCode::Backspace => {
                        if !autocomplete.pop_char(&field_names) {
                            // Empty, go back to Default state
                            self.state = FilterState::Default {
                                autocomplete: AutocompleteState::default(),
                                conditions: conditions.clone(),
                            };
                        }
                        true
                    }
                    KeyCode::Tab => {
                        autocomplete.cycle_next();
                        true
                    }
                    KeyCode::BackTab => {
                        autocomplete.cycle_prev();
                        true
                    }
                    KeyCode::Char(c) => {
                        autocomplete.push_char(c, &field_names);
                        true
                    }
                    _ => false,
                }
            }

            FilterState::ValueInput { field, field_kind, autocomplete, conditions } => {
                let value_candidates = field_kind.values();

                match key.code {
                    KeyCode::Esc | KeyCode::Enter => {
                        self.deactivate();
                        true
                    }
                    KeyCode::Char(':') if self.space_just_pressed => {
                        // Chain to new attribute selection
                        // First, confirm current value if any
                        let mut new_conditions = conditions.clone();
                        if !autocomplete.typed.is_empty() {
                            new_conditions.push(FilterCondition::new(
                                field.clone(),
                                &autocomplete.typed,
                                false,
                            ));
                        }

                        let field_names: Vec<String> = fields
                            .iter()
                            .filter(|f| !f.is_primary)
                            .map(|f| f.name.to_string())
                            .collect();

                        let mut new_autocomplete = AutocompleteState::default();
                        new_autocomplete.update_candidates(&field_names);

                        self.state = FilterState::AttributeSelection {
                            autocomplete: new_autocomplete,
                            conditions: new_conditions,
                        };
                        self.space_just_pressed = false;
                        true
                    }
                    KeyCode::Char(' ') => {
                        // Confirm current filter and go back to Default
                        let mut new_conditions = conditions.clone();
                        if !autocomplete.typed.is_empty() {
                            new_conditions.push(FilterCondition::new(
                                field.clone(),
                                &autocomplete.typed,
                                false,
                            ));
                        }

                        self.state = FilterState::Default {
                            autocomplete: AutocompleteState::default(),
                            conditions: new_conditions,
                        };
                        self.space_just_pressed = true;
                        true
                    }
                    KeyCode::Backspace => {
                        self.space_just_pressed = false;
                        if !autocomplete.pop_char(&value_candidates) {
                            // Empty, go back to AttributeSelection
                            let field_names: Vec<String> = fields
                                .iter()
                                .filter(|f| !f.is_primary)
                                .map(|f| f.name.to_string())
                                .collect();

                            let mut new_autocomplete = AutocompleteState::default();
                            new_autocomplete.typed = field.clone();
                            new_autocomplete.update_candidates(&field_names);

                            self.state = FilterState::AttributeSelection {
                                autocomplete: new_autocomplete,
                                conditions: conditions.clone(),
                            };
                        }
                        true
                    }
                    KeyCode::Tab => {
                        self.space_just_pressed = false;
                        autocomplete.cycle_next();
                        true
                    }
                    KeyCode::BackTab => {
                        self.space_just_pressed = false;
                        autocomplete.cycle_prev();
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

    /// Get values for primary field autocomplete (empty for now, could be populated from data)
    fn get_primary_values(_fields: &[FilterableField]) -> Vec<String> {
        // For primary field, we don't have predefined values
        // Could be extended to use actual data values
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    fn test_fields() -> Vec<FilterableField> {
        vec![
            FilterableField::primary("dag_run_id"),
            FilterableField::enumerated("state", vec!["running", "success", "failed"]),
            FilterableField::enumerated("run_type", vec!["scheduled", "manual"]),
        ]
    }

    #[test]
    fn test_activate_on_slash() {
        let mut fsm = FilterStateMachine::new();
        let fields = test_fields();

        assert!(!fsm.is_active());
        assert!(fsm.update(&key(KeyCode::Char('/')), &fields));
        assert!(fsm.is_active());
    }

    #[test]
    fn test_deactivate_on_esc() {
        let mut fsm = FilterStateMachine::new();
        let fields = test_fields();

        fsm.activate();
        assert!(fsm.update(&key(KeyCode::Esc), &fields));
        assert!(!fsm.is_active());
    }

    #[test]
    fn test_type_in_default_mode() {
        let mut fsm = FilterStateMachine::new();
        let fields = test_fields();

        fsm.activate();
        fsm.update(&key(KeyCode::Char('m')), &fields);
        fsm.update(&key(KeyCode::Char('y')), &fields);

        let conditions = fsm.active_conditions();
        assert_eq!(conditions.len(), 1);
        assert!(conditions[0].is_primary);
        assert_eq!(conditions[0].value, "my");
    }

    #[test]
    fn test_transition_to_attribute_selection() {
        let mut fsm = FilterStateMachine::new();
        let fields = test_fields();

        fsm.activate();
        fsm.update(&key(KeyCode::Char(':')), &fields);

        assert!(matches!(fsm.state, FilterState::AttributeSelection { .. }));
    }

    #[test]
    fn test_attribute_autocomplete() {
        let mut fsm = FilterStateMachine::new();
        let fields = test_fields();

        fsm.activate();
        fsm.update(&key(KeyCode::Char(':')), &fields);
        fsm.update(&key(KeyCode::Char('s')), &fields);

        if let FilterState::AttributeSelection { autocomplete, .. } = &fsm.state {
            assert_eq!(autocomplete.typed, "s");
            assert_eq!(autocomplete.candidates, vec!["state"]);
        } else {
            panic!("Expected AttributeSelection state");
        }
    }

    #[test]
    fn test_confirm_attribute_and_enter_value() {
        let mut fsm = FilterStateMachine::new();
        let fields = test_fields();

        fsm.activate();
        fsm.update(&key(KeyCode::Char(':')), &fields);
        fsm.update(&key(KeyCode::Char('s')), &fields);
        fsm.update(&key(KeyCode::Char('t')), &fields);
        fsm.update(&key(KeyCode::Char('a')), &fields);
        fsm.update(&key(KeyCode::Char(' ')), &fields); // Confirm "state"

        if let FilterState::ValueInput { field, autocomplete, .. } = &fsm.state {
            assert_eq!(field, "state");
            // Should have enum values as candidates
            assert!(autocomplete.candidates.contains(&"running".to_string()));
        } else {
            panic!("Expected ValueInput state, got {:?}", fsm.state);
        }
    }

    #[test]
    fn test_value_input_with_enum() {
        let mut fsm = FilterStateMachine::new();
        let fields = test_fields();

        fsm.activate();
        fsm.update(&key(KeyCode::Char(':')), &fields);
        fsm.update(&key(KeyCode::Char('s')), &fields);
        fsm.update(&key(KeyCode::Char('t')), &fields);
        fsm.update(&key(KeyCode::Char('a')), &fields);
        fsm.update(&key(KeyCode::Char(' ')), &fields); // Confirm "state"
        fsm.update(&key(KeyCode::Char('r')), &fields);

        if let FilterState::ValueInput { autocomplete, .. } = &fsm.state {
            assert_eq!(autocomplete.typed, "r");
            assert_eq!(autocomplete.candidates, vec!["running"]);
        } else {
            panic!("Expected ValueInput state");
        }

        // Check active conditions include the in-progress filter
        let conditions = fsm.active_conditions();
        assert_eq!(conditions.len(), 1);
        assert_eq!(conditions[0].field, "state");
        assert_eq!(conditions[0].value, "r");
    }

    #[test]
    fn test_chain_filters() {
        let mut fsm = FilterStateMachine::new();
        let fields = test_fields();

        // First filter: state = running
        fsm.activate();
        fsm.update(&key(KeyCode::Char(':')), &fields);
        fsm.update(&key(KeyCode::Char('s')), &fields);
        fsm.update(&key(KeyCode::Char('t')), &fields);
        fsm.update(&key(KeyCode::Char('a')), &fields);
        fsm.update(&key(KeyCode::Char(' ')), &fields); // Confirm "state"
        fsm.update(&key(KeyCode::Char('r')), &fields);
        fsm.update(&key(KeyCode::Char('u')), &fields);
        fsm.update(&key(KeyCode::Char('n')), &fields);
        fsm.update(&key(KeyCode::Char(' ')), &fields); // Confirm value, back to Default

        // Should be back in Default with one confirmed condition
        assert!(matches!(fsm.state, FilterState::Default { .. }));
        assert_eq!(fsm.state.confirmed_conditions().len(), 1);

        // Chain another filter
        fsm.update(&key(KeyCode::Char(':')), &fields);

        assert!(matches!(fsm.state, FilterState::AttributeSelection { .. }));
        assert_eq!(fsm.state.confirmed_conditions().len(), 1);
    }

    #[test]
    fn test_backspace_transitions() {
        let mut fsm = FilterStateMachine::new();
        let fields = test_fields();

        // Go to ValueInput
        fsm.activate();
        fsm.update(&key(KeyCode::Char(':')), &fields);
        fsm.update(&key(KeyCode::Char('s')), &fields);
        fsm.update(&key(KeyCode::Char(' ')), &fields);
        fsm.update(&key(KeyCode::Char('r')), &fields);

        assert!(matches!(fsm.state, FilterState::ValueInput { .. }));

        // Backspace once removes 'r'
        fsm.update(&key(KeyCode::Backspace), &fields);
        assert!(matches!(fsm.state, FilterState::ValueInput { .. }));

        // Backspace again goes back to AttributeSelection
        fsm.update(&key(KeyCode::Backspace), &fields);
        assert!(matches!(fsm.state, FilterState::AttributeSelection { .. }));
    }

    #[test]
    fn test_tab_cycling() {
        let mut fsm = FilterStateMachine::new();
        let fields = test_fields();

        fsm.activate();
        fsm.update(&key(KeyCode::Char(':')), &fields);

        // Should have "state" and "run_type" as candidates
        if let FilterState::AttributeSelection { autocomplete, .. } = &fsm.state {
            assert_eq!(autocomplete.selected_index, 0);
        }

        fsm.update(&key(KeyCode::Tab), &fields);

        if let FilterState::AttributeSelection { autocomplete, .. } = &fsm.state {
            assert_eq!(autocomplete.selected_index, 1);
        }

        // Tab again wraps
        fsm.update(&key(KeyCode::Tab), &fields);

        if let FilterState::AttributeSelection { autocomplete, .. } = &fsm.state {
            assert_eq!(autocomplete.selected_index, 0);
        }
    }
}
```

**Step 2: Update mod.rs**

Add to `src/app/model/filter/mod.rs`:

```rust
mod state_machine;
pub use state_machine::FilterStateMachine;
```

**Step 3: Run tests**

Run: `cargo test state_machine`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/app/model/filter/state_machine.rs src/app/model/filter/mod.rs
git commit -m "feat(filter): add FilterStateMachine with keyboard handling"
```

---

## Task 6: Implement Filterable for DagRun

**Files:**
- Modify: `src/airflow/model/common/dagrun.rs`

**Step 1: Add Filterable implementation**

Add at the end of `src/airflow/model/common/dagrun.rs`:

```rust
use crate::app::model::filter::{Filterable, FilterableField, FilterKind};

impl Filterable for DagRun {
    fn primary_field() -> &'static str {
        "dag_run_id"
    }

    fn filterable_fields() -> Vec<FilterableField> {
        vec![
            FilterableField::primary("dag_run_id"),
            FilterableField::enumerated(
                "state",
                vec!["running", "success", "failed", "queued", "up_for_retry"],
            ),
            FilterableField::enumerated(
                "run_type",
                vec!["scheduled", "manual", "backfill", "dataset_triggered"],
            ),
        ]
    }

    fn get_field_value(&self, field_name: &str) -> Option<String> {
        match field_name {
            "dag_run_id" => Some(self.dag_run_id.clone()),
            "dag_id" => Some(self.dag_id.clone()),
            "state" => Some(self.state.clone()),
            "run_type" => Some(self.run_type.clone()),
            _ => None,
        }
    }
}
```

**Step 2: Run check**

Run: `cargo check`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add src/airflow/model/common/dagrun.rs
git commit -m "feat(filter): implement Filterable for DagRun"
```

---

## Task 7: Implement Filterable for Dag

**Files:**
- Modify: `src/airflow/model/common/dag.rs`

**Step 1: Add Filterable implementation**

Add at the end of `src/airflow/model/common/dag.rs`:

```rust
use crate::app::model::filter::{Filterable, FilterableField};

impl Filterable for Dag {
    fn primary_field() -> &'static str {
        "dag_id"
    }

    fn filterable_fields() -> Vec<FilterableField> {
        vec![
            FilterableField::primary("dag_id"),
            FilterableField::enumerated("is_paused", vec!["true", "false"]),
            FilterableField::free_text("owners"),
            FilterableField::free_text("tags"),
        ]
    }

    fn get_field_value(&self, field_name: &str) -> Option<String> {
        match field_name {
            "dag_id" => Some(self.dag_id.clone()),
            "is_paused" => Some(self.is_paused.to_string()),
            "owners" => Some(self.owners.join(", ")),
            "tags" => Some(self.tags.iter().map(|t| t.name.clone()).collect::<Vec<_>>().join(", ")),
            "description" => self.description.clone(),
            _ => None,
        }
    }
}
```

**Step 2: Run check**

Run: `cargo check`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add src/airflow/model/common/dag.rs
git commit -m "feat(filter): implement Filterable for Dag"
```

---

## Task 8: Implement Filterable for TaskInstance

**Files:**
- Modify: `src/airflow/model/common/taskinstance.rs`

**Step 1: Add Filterable implementation**

Add at the end of `src/airflow/model/common/taskinstance.rs`:

```rust
use crate::app::model::filter::{Filterable, FilterableField, FilterKind};

impl Filterable for TaskInstance {
    fn primary_field() -> &'static str {
        "task_id"
    }

    fn filterable_fields() -> Vec<FilterableField> {
        vec![
            FilterableField::primary("task_id"),
            FilterableField::enumerated(
                "state",
                vec![
                    "running",
                    "success",
                    "failed",
                    "queued",
                    "up_for_retry",
                    "up_for_reschedule",
                    "skipped",
                    "deferred",
                    "removed",
                    "restarting",
                ],
            ),
            FilterableField::free_text("operator"),
        ]
    }

    fn get_field_value(&self, field_name: &str) -> Option<String> {
        match field_name {
            "task_id" => Some(self.task_id.clone()),
            "dag_id" => Some(self.dag_id.clone()),
            "state" => self.state.clone(),
            "operator" => self.operator.clone(),
            _ => None,
        }
    }
}
```

**Step 2: Run check**

Run: `cargo check`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add src/airflow/model/common/taskinstance.rs
git commit -m "feat(filter): implement Filterable for TaskInstance"
```

---

## Task 9: Implement Filterable for AirflowConfig

**Files:**
- Modify: `src/airflow/config/mod.rs`

**Step 1: Add Filterable implementation**

Add at the end of `src/airflow/config/mod.rs` (before the tests module):

```rust
use crate::app::model::filter::{Filterable, FilterableField};

impl Filterable for AirflowConfig {
    fn primary_field() -> &'static str {
        "name"
    }

    fn filterable_fields() -> Vec<FilterableField> {
        vec![FilterableField::primary("name")]
    }

    fn get_field_value(&self, field_name: &str) -> Option<String> {
        match field_name {
            "name" => Some(self.name.clone()),
            "endpoint" => Some(self.endpoint.clone()),
            _ => None,
        }
    }
}
```

**Step 2: Run check**

Run: `cargo check`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add src/airflow/config/mod.rs
git commit -m "feat(filter): implement Filterable for AirflowConfig"
```

---

## Task 10: Create Generic Matching Function

**Files:**
- Create: `src/app/model/filter/matching.rs`
- Modify: `src/app/model/filter/mod.rs`

**Step 1: Write the matching function with tests**

Create `src/app/model/filter/matching.rs`:

```rust
use super::{FilterCondition, Filterable};

/// Check if an item matches all filter conditions
pub fn matches<T: Filterable>(item: &T, conditions: &[FilterCondition]) -> bool {
    conditions.iter().all(|cond| {
        let field_name = if cond.is_primary {
            T::primary_field()
        } else {
            &cond.field
        };

        item.get_field_value(field_name)
            .map(|v| cond.matches(&v))
            .unwrap_or(false)
    })
}

/// Filter a collection of items by conditions
pub fn filter_items<T: Filterable + Clone>(items: &[T], conditions: &[FilterCondition]) -> Vec<T> {
    if conditions.is_empty() {
        return items.to_vec();
    }

    items
        .iter()
        .filter(|item| matches(*item, conditions))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::model::filter::FilterableField;

    // Test struct
    #[derive(Clone, Debug)]
    struct TestItem {
        id: String,
        status: String,
    }

    impl Filterable for TestItem {
        fn primary_field() -> &'static str {
            "id"
        }

        fn filterable_fields() -> Vec<FilterableField> {
            vec![
                FilterableField::primary("id"),
                FilterableField::enumerated("status", vec!["active", "inactive"]),
            ]
        }

        fn get_field_value(&self, field_name: &str) -> Option<String> {
            match field_name {
                "id" => Some(self.id.clone()),
                "status" => Some(self.status.clone()),
                _ => None,
            }
        }
    }

    #[test]
    fn test_matches_primary() {
        let item = TestItem {
            id: "my_item_123".to_string(),
            status: "active".to_string(),
        };

        let cond = FilterCondition::primary("item");
        assert!(matches(&item, &[cond]));

        let cond = FilterCondition::primary("other");
        assert!(!matches(&item, &[cond]));
    }

    #[test]
    fn test_matches_field() {
        let item = TestItem {
            id: "my_item".to_string(),
            status: "active".to_string(),
        };

        let cond = FilterCondition::new("status", "act", false);
        assert!(matches(&item, &[cond]));

        let cond = FilterCondition::new("status", "inactive", false);
        assert!(!matches(&item, &[cond]));
    }

    #[test]
    fn test_matches_multiple_conditions() {
        let item = TestItem {
            id: "my_item".to_string(),
            status: "active".to_string(),
        };

        let conditions = vec![
            FilterCondition::primary("my"),
            FilterCondition::new("status", "active", false),
        ];
        assert!(matches(&item, &conditions));

        let conditions = vec![
            FilterCondition::primary("my"),
            FilterCondition::new("status", "inactive", false),
        ];
        assert!(!matches(&item, &conditions)); // Second doesn't match
    }

    #[test]
    fn test_filter_items() {
        let items = vec![
            TestItem { id: "item_1".to_string(), status: "active".to_string() },
            TestItem { id: "item_2".to_string(), status: "inactive".to_string() },
            TestItem { id: "other_3".to_string(), status: "active".to_string() },
        ];

        // Filter by primary
        let conditions = vec![FilterCondition::primary("item")];
        let filtered = filter_items(&items, &conditions);
        assert_eq!(filtered.len(), 2);

        // Filter by status
        let conditions = vec![FilterCondition::new("status", "active", false)];
        let filtered = filter_items(&items, &conditions);
        assert_eq!(filtered.len(), 2);

        // Combined
        let conditions = vec![
            FilterCondition::primary("item"),
            FilterCondition::new("status", "active", false),
        ];
        let filtered = filter_items(&items, &conditions);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "item_1");
    }

    #[test]
    fn test_empty_conditions() {
        let items = vec![
            TestItem { id: "item_1".to_string(), status: "active".to_string() },
        ];

        let filtered = filter_items(&items, &[]);
        assert_eq!(filtered.len(), 1);
    }
}
```

**Step 2: Update mod.rs**

Add to `src/app/model/filter/mod.rs`:

```rust
mod matching;
pub use matching::{matches, filter_items};
```

**Step 3: Run tests**

Run: `cargo test matching`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/app/model/filter/matching.rs src/app/model/filter/mod.rs
git commit -m "feat(filter): add generic matching functions"
```

---

## Task 11: Create Filter Widget for Rendering

**Files:**
- Create: `src/app/model/filter/widget.rs`
- Modify: `src/app/model/filter/mod.rs`

**Step 1: Write the widget**

Create `src/app/model/filter/widget.rs`:

```rust
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

use super::{FilterState, FilterStateMachine};
use crate::ui::constants::DEFAULT_STYLE;
use crate::ui::theme::ACCENT;

/// Cursor state for the filter widget
#[derive(Clone, Default, Debug)]
pub struct FilterCursor {
    pub position: Position,
}

impl FilterStateMachine {
    /// Render the filter widget and return cursor position
    pub fn render_widget(&self, area: Rect, buf: &mut Buffer) -> Option<Position> {
        if !self.is_active() {
            return None;
        }

        let (content, cursor_offset) = self.build_display_content();

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .title(self.title())
                    .style(DEFAULT_STYLE),
            )
            .style(DEFAULT_STYLE);

        Widget::render(paragraph, area, buf);

        // Calculate cursor position
        Some(Position {
            x: area.x + 1 + cursor_offset,
            y: area.y + 1,
        })
    }

    fn title(&self) -> &'static str {
        match &self.state {
            FilterState::Inactive => "filter",
            FilterState::Default { .. } => "filter",
            FilterState::AttributeSelection { .. } => "filter (select attribute)",
            FilterState::ValueInput { field, .. } => "filter",
        }
    }

    fn build_display_content(&self) -> (Line<'static>, u16) {
        match &self.state {
            FilterState::Inactive => (Line::from(""), 0),

            FilterState::Default { autocomplete, conditions } => {
                let mut spans = Vec::new();
                let mut cursor_offset: u16 = 0;

                // Render confirmed conditions
                for cond in conditions {
                    let cond_text = format!("{}: {} ", cond.field, cond.value);
                    cursor_offset += cond_text.len() as u16;
                    spans.push(Span::styled(cond_text, Style::default().fg(Color::Gray)));
                }

                // Render typed text
                let typed = &autocomplete.typed;
                cursor_offset += typed.len() as u16;
                spans.push(Span::styled(typed.clone(), Style::default().add_modifier(Modifier::BOLD)));

                // Render ghost text
                if let Some(ghost) = autocomplete.ghost_suffix() {
                    spans.push(Span::styled(ghost.to_string(), Style::default().fg(Color::DarkGray)));
                }

                (Line::from(spans), cursor_offset)
            }

            FilterState::AttributeSelection { autocomplete, conditions } => {
                let mut spans = Vec::new();
                let mut cursor_offset: u16 = 0;

                // Render confirmed conditions
                for cond in conditions {
                    let cond_text = format!("{}: {} ", cond.field, cond.value);
                    cursor_offset += cond_text.len() as u16;
                    spans.push(Span::styled(cond_text, Style::default().fg(Color::Gray)));
                }

                // Render colon prefix
                spans.push(Span::styled(":", Style::default().fg(ACCENT)));
                cursor_offset += 1;

                // Render typed attribute name
                let typed = &autocomplete.typed;
                cursor_offset += typed.len() as u16;
                spans.push(Span::styled(typed.clone(), Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)));

                // Render ghost text for attribute
                if let Some(ghost) = autocomplete.ghost_suffix() {
                    spans.push(Span::styled(ghost.to_string(), Style::default().fg(Color::DarkGray)));
                }

                (Line::from(spans), cursor_offset)
            }

            FilterState::ValueInput { field, autocomplete, conditions, .. } => {
                let mut spans = Vec::new();
                let mut cursor_offset: u16 = 0;

                // Render confirmed conditions
                for cond in conditions {
                    let cond_text = format!("{}: {} ", cond.field, cond.value);
                    cursor_offset += cond_text.len() as u16;
                    spans.push(Span::styled(cond_text, Style::default().fg(Color::Gray)));
                }

                // Render field name with colon
                let field_prefix = format!("{}: ", field);
                cursor_offset += field_prefix.len() as u16;
                spans.push(Span::styled(field_prefix, Style::default().fg(ACCENT)));

                // Render typed value
                let typed = &autocomplete.typed;
                cursor_offset += typed.len() as u16;
                spans.push(Span::styled(typed.clone(), Style::default().add_modifier(Modifier::BOLD)));

                // Render ghost text for value
                if let Some(ghost) = autocomplete.ghost_suffix() {
                    spans.push(Span::styled(ghost.to_string(), Style::default().fg(Color::DarkGray)));
                }

                (Line::from(spans), cursor_offset)
            }
        }
    }
}
```

**Step 2: Update mod.rs**

Add to `src/app/model/filter/mod.rs`:

```rust
mod widget;
pub use widget::FilterCursor;
```

**Step 3: Run check**

Run: `cargo check`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add src/app/model/filter/widget.rs src/app/model/filter/mod.rs
git commit -m "feat(filter): add widget rendering with ghost text"
```

---

## Task 12: Finalize Filter Module Structure

**Files:**
- Modify: `src/app/model/filter/mod.rs`

**Step 1: Clean up mod.rs with final exports**

Update `src/app/model/filter/mod.rs` to be the final version:

```rust
//! Filter state machine with autocomplete support
//!
//! This module provides a filter state machine that supports:
//! - Autocomplete with inline ghost text
//! - Multiple attribute filtering with AND logic
//! - Enum-aware value completion

mod autocomplete;
mod condition;
mod filterable;
mod matching;
mod state;
mod state_machine;
mod widget;

// Re-export legacy Filter for backwards compatibility during migration
mod legacy;
pub use legacy::{CursorState, Filter};

// New filter system exports
pub use autocomplete::AutocompleteState;
pub use condition::FilterCondition;
pub use filterable::{FilterKind, FilterableField, Filterable};
pub use matching::{filter_items, matches};
pub use state::FilterState;
pub use state_machine::FilterStateMachine;
pub use widget::FilterCursor;
```

**Step 2: Run all filter tests**

Run: `cargo test filter`
Expected: All tests pass

**Step 3: Run full test suite**

Run: `cargo test`
Expected: All tests pass (including existing tests)

**Step 4: Commit**

```bash
git add src/app/model/filter/mod.rs
git commit -m "feat(filter): finalize module structure with all exports"
```

---

## Task 13: Integrate FilterStateMachine into DagRunModel

**Files:**
- Modify: `src/app/model/dagruns.rs`

**Step 1: Update imports and struct**

At the top of `src/app/model/dagruns.rs`, update the filter import:

```rust
// Change this line:
use super::{filter::Filter, Model, StatefulTable};

// To:
use super::{
    filter::{Filter, FilterStateMachine, Filterable, filter_items},
    Model, StatefulTable,
};
```

**Step 2: Add new filter field alongside old one**

In the `DagRunModel` struct, add:

```rust
pub filter_v2: FilterStateMachine,
```

Keep the old `filter: Filter` for now to maintain backwards compatibility during migration.

**Step 3: Update the filter method**

Add a new method `filter_dag_runs_v2`:

```rust
pub fn filter_dag_runs_v2(&mut self) {
    let conditions = self.filter_v2.active_conditions();
    let mut filtered = filter_items(&self.all, &conditions);

    // Apply sorting by logical_date descending
    filtered.sort_by(|a, b| {
        b.logical_date
            .or(b.start_date)
            .cmp(&a.logical_date.or(a.start_date))
    });

    self.filtered.items = filtered;
}
```

**Step 4: Update keyboard handling in update()**

In the `update` method, add handling for the new filter before the old one:

```rust
// Near the top of the match, add:
if self.filter_v2.is_active() {
    if self.filter_v2.update(key_event, &DagRun::filterable_fields()) {
        self.filter_dag_runs_v2();
        return (None, vec![]);
    }
}
```

**Step 5: Run check**

Run: `cargo check`
Expected: Compiles without errors

**Step 6: Commit**

```bash
git add src/app/model/dagruns.rs
git commit -m "feat(filter): integrate FilterStateMachine into DagRunModel"
```

---

## Task 14: Update UI Rendering for DagRunModel

**Files:**
- Modify: `src/app/model/dagruns.rs` (render-related methods)

**Step 1: Find and update the render logic**

Locate the UI rendering code in `dagruns.rs` that handles the filter display. Update it to use the new widget when `filter_v2` is active.

The key changes:
1. Check `self.filter_v2.is_active()` instead of `self.filter.is_enabled()`
2. Use `self.filter_v2.render_widget(filter_area, buf)` for rendering
3. Handle cursor positioning from the new widget

**Step 2: Run the app to test**

Run: `cargo run`
Expected: App launches, press `/` to activate filter, test autocomplete with `:state`

**Step 3: Commit**

```bash
git add src/app/model/dagruns.rs
git commit -m "feat(filter): update DagRunModel UI to use new filter widget"
```

---

## Task 15: Migrate Remaining Panels (Config, Dags, TaskInstances)

**Files:**
- Modify: `src/app/model/config.rs`
- Modify: `src/app/model/dags.rs`
- Modify: `src/app/model/taskinstances.rs`

Follow the same pattern as Task 14-15 for each panel:

1. Update imports to include new filter types
2. Add `filter_v2: FilterStateMachine` field
3. Add `filter_*_v2()` method using `filter_items()`
4. Update keyboard handling in `update()`
5. Update UI rendering

**Step 1: Update ConfigModel**

Apply changes to `src/app/model/config.rs`

**Step 2: Update DagModel**

Apply changes to `src/app/model/dags.rs`

**Step 3: Update TaskInstanceModel**

Apply changes to `src/app/model/taskinstances.rs`

**Step 4: Run full test suite**

Run: `cargo test`
Expected: All tests pass

**Step 5: Run the app and test all panels**

Run: `cargo run`
Test each panel's filter functionality

**Step 6: Commit**

```bash
git add src/app/model/config.rs src/app/model/dags.rs src/app/model/taskinstances.rs
git commit -m "feat(filter): migrate all panels to FilterStateMachine"
```

---

## Task 16: Remove Legacy Filter

**Files:**
- Delete: `src/app/model/filter/legacy.rs`
- Modify: `src/app/model/filter/mod.rs`
- Modify: All panel files to remove old filter field

**Step 1: Remove legacy re-exports from mod.rs**

Remove these lines from `src/app/model/filter/mod.rs`:

```rust
mod legacy;
pub use legacy::{CursorState, Filter};
```

**Step 2: Remove old filter field from all models**

In each model file, remove:
- `pub filter: Filter,` field
- `filter_*()` old method (keep only `filter_*_v2()`)
- Any code referencing the old filter

**Step 3: Rename filter_v2 to filter**

In each model, rename `filter_v2` to `filter` and `filter_*_v2()` to `filter_*()`

**Step 4: Delete legacy.rs**

Remove `src/app/model/filter/legacy.rs`

**Step 5: Run full test suite**

Run: `cargo test`
Expected: All tests pass

**Step 6: Run clippy**

Run: `cargo clippy`
Expected: No warnings related to filter

**Step 7: Commit**

```bash
git add -A
git commit -m "refactor(filter): remove legacy Filter, complete migration"
```

---

## Task 17: Final Integration Testing

**Files:** None (testing only)

**Step 1: Manual testing checklist**

Test each panel:

- [ ] Config panel: `/` activates filter, typing filters by name
- [ ] DAGs panel: `/` activates, `:is_paused true` works
- [ ] DAG Runs panel: `/` activates, `:state running` works, chaining `:run_type manual` works
- [ ] Task Instances panel: `/` activates, `:state failed` works

Test autocomplete:
- [ ] Tab cycles through candidates
- [ ] Ghost text appears after typing partial match
- [ ] Backspace navigates back through states
- [ ] Esc/Enter closes filter

Test chaining:
- [ ] Can add multiple filters with space + `:`
- [ ] All filters apply (AND logic)
- [ ] Live filtering updates results immediately

**Step 2: Run full test suite**

Run: `cargo test`
Expected: All tests pass

**Step 3: Build release**

Run: `cargo build --release`
Expected: Builds successfully

**Step 4: Final commit**

```bash
git add -A
git commit -m "feat(filter): complete filter state machine implementation"
```

---

## Summary

This plan implements:
1. **AutocompleteState** - Ghost text and Tab cycling
2. **FilterCondition** - Individual filter criteria
3. **Filterable trait** - Reflection for field discovery
4. **FilterState enum** - State machine states
5. **FilterStateMachine** - Keyboard handling and transitions
6. **Generic matching** - `filter_items()` function
7. **Widget rendering** - Ghost text display
8. **Panel integration** - All 4 panels migrated
9. **Legacy cleanup** - Old filter removed

Total: 17 tasks, each independently testable and committable.
