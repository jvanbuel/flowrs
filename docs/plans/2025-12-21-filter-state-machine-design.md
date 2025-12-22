# Filter State Machine Design

This document describes the design for a filter state machine with autocomplete functionality for the Flowrs TUI application.

## Overview

Transform the simple substring filter into a state machine that supports:
- Autocomplete with inline ghost text
- Filtering on multiple attributes (not just the primary identifier)
- Chaining multiple filters with AND logic
- Trait-based field discovery via the `Filterable` trait

## State Model

```
┌─────────────┐   '/'    ┌─────────────┐   ':'    ┌─────────────┐
│   Inactive  │ ──────▶  │  Default    │ ──────▶  │  Attribute  │
│             │          │  (by name)  │          │  Selection  │
└─────────────┘          └─────────────┘          └─────────────┘
       ▲                        ▲    ▲                   │
       │ Esc/Enter              │    │ ':'               │ Space
       │                        │    │                   ▼
       └────────────────────────┴────┴───────────┌─────────────┐
                                                 │   Value     │
                                        Space    │   Input     │
                                       (+ ':')   └─────────────┘
```

### States

- **Inactive**: Filter UI hidden, normal table navigation
- **Default**: Filtering live on primary field as user types
- **AttributeSelection**: After `:`, selecting field name with autocomplete
- **ValueInput**: Attribute confirmed, filtering live on that field as user types

### Transitions

| From | Trigger | To |
|------|---------|-----|
| Inactive | `/` | Default |
| Default | `:` | AttributeSelection |
| AttributeSelection | `Space` | ValueInput |
| ValueInput | `Space` | Default (filter confirmed, can add more) |
| ValueInput | `:` after `Space` | AttributeSelection (chaining) |
| Any active state | `Enter` or `Esc` | Inactive |

### Key Behaviors

- Filtering is **always live** - results update on every keystroke
- Multiple filters accumulate and are AND'd together
- `Tab` cycles through autocomplete candidates
- `Backspace` on empty input goes back to previous state

## Autocomplete Mechanism

### State Tracking

```rust
pub struct AutocompleteState {
    pub typed: String,           // What user actually typed (e.g., "sta")
    pub candidates: Vec<String>, // Matching options (e.g., ["state", "status"])
    pub selected_index: usize,   // Current selection for Tab cycling
}
```

### Ghost Text Display

- Display format: `typed` + grayed-out suffix from `candidates[selected_index]`
- Example: User typed "sta", candidates are ["state", "status"], index 0
  - Renders: `sta|te` where "te" is dimmed/grayed

### Tab Behavior

- Increments `selected_index` (wrapping)
- Ghost text updates to show next candidate's suffix
- `typed` remains unchanged (filtering candidates stays consistent)

### Space Behavior (in AttributeSelection)

- Confirms `candidates[selected_index]` as the chosen attribute
- Transitions to ValueInput state
- Clears autocomplete state for value entry

## Filterable Trait

### FilterKind and FilterableField

```rust
#[derive(Debug, Clone)]
pub enum FilterKind {
    FreeText,
    Enum(Vec<&'static str>),
}

pub struct FilterableField {
    pub name: &'static str,
    pub kind: FilterKind,
    pub is_primary: bool,
}
```

### Filterable Trait

```rust
pub trait Filterable {
    fn primary_field() -> &'static str;
    fn filterable_fields() -> Vec<FilterableField>;
    fn get_field_value(&self, field_name: &str) -> Option<String>;
}
```

### Example Implementation

```rust
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
            "state" => Some(self.state.clone()),
            "run_type" => Some(self.run_type.clone()),
            _ => None,
        }
    }
}
```

## Core Data Structures

```rust
#[derive(Clone, Debug)]
pub struct FilterCondition {
    pub field: String,
    pub value: String,
    pub is_primary: bool,
}

#[derive(Clone, Debug)]
pub enum FilterState {
    Inactive,
    Default {
        autocomplete: AutocompleteState,
        conditions: Vec<FilterCondition>,
    },
    AttributeSelection {
        autocomplete: AutocompleteState,
        conditions: Vec<FilterCondition>,
    },
    ValueInput {
        field: String,
        field_kind: FilterKind,
        autocomplete: AutocompleteState,
        conditions: Vec<FilterCondition>,
    },
}
```

## Matching Logic

```rust
impl FilterState {
    pub fn active_conditions(&self) -> Vec<FilterCondition> {
        match self {
            FilterState::Inactive => vec![],
            FilterState::Default { autocomplete, conditions } => {
                let mut result = conditions.clone();
                if !autocomplete.typed.is_empty() {
                    result.push(FilterCondition {
                        field: "<primary>".into(),
                        value: autocomplete.typed.clone(),
                        is_primary: true,
                    });
                }
                result
            }
            FilterState::AttributeSelection { conditions, .. } => {
                conditions.clone()
            }
            FilterState::ValueInput { field, autocomplete, conditions, .. } => {
                let mut result = conditions.clone();
                if !autocomplete.typed.is_empty() {
                    result.push(FilterCondition {
                        field: field.clone(),
                        value: autocomplete.typed.clone(),
                        is_primary: false,
                    });
                }
                result
            }
        }
    }
}

pub fn matches<T: Filterable>(item: &T, conditions: &[FilterCondition]) -> bool {
    conditions.iter().all(|cond| {
        let field_value = if cond.is_primary {
            get_field_value(item, T::primary_field())
        } else {
            get_field_value(item, &cond.field)
        };

        field_value
            .map(|v| v.to_lowercase().contains(&cond.value.to_lowercase()))
            .unwrap_or(false)
    })
}
```

## UI Rendering

### Display Format

```
┌─ filter ─────────────────────────────────────────────────────┐
│ status: running  dag_run_id: my_da|g_run_123                 │
│                                   ▲                          │
│                              ghost text                      │
└──────────────────────────────────────────────────────────────┘
```

### Visual Styling

- Confirmed filters: normal text color
- Current typed text: bright/highlighted
- Ghost text (autocomplete suggestion): dim gray
- Attribute names: accent color (cyan/blue)
- Separators (`:` and spaces): muted

## Keyboard Handling

| Key | Inactive | Default | AttributeSelection | ValueInput |
|-----|----------|---------|-------------------|------------|
| `/` | → Default | (literal) | (literal) | (literal) |
| `:` | - | → AttrSelection | (literal) | → AttrSelection* |
| `Space` | - | (literal) | confirm → ValueInput | confirm → Default |
| `Tab` | - | cycle | cycle | cycle |
| `Backspace` | - | delete | delete (or → Default) | delete (or → AttrSelection) |
| `Enter` | - | → Inactive | → Inactive | → Inactive |
| `Esc` | - | → Inactive | → Inactive | → Inactive |
| `Char(c)` | - | append | append | append |

*Only if preceded by Space (chaining filters)

## Panel Integration

### Module Structure

```
src/app/model/filter/
├── mod.rs              # Re-exports
├── state_machine.rs    # FilterStateMachine, FilterState
├── autocomplete.rs     # AutocompleteState
├── condition.rs        # FilterCondition
├── matching.rs         # Generic matching logic
├── widget.rs           # UI rendering
└── filterable.rs       # Filterable trait, derive macro helpers
```

### Model Changes

```rust
// Before:
pub filter: Filter,

// After:
pub filter: FilterStateMachine<DagRun>,
```

### Integration Pattern

```rust
impl DagRunModel {
    pub fn update(&mut self, event: FlowrsEvent) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        if let FlowrsEvent::Key(key_event) = &event {
            if self.filter.update(key_event, &DagRun::filterable_fields()) {
                self.apply_filter();
                return (None, vec![]);
            }
        }
        // ...
    }

    fn apply_filter(&mut self) {
        let conditions = self.filter.active_conditions();
        self.filtered.items = self.all
            .iter()
            .filter(|item| matches(item, &conditions))
            .cloned()
            .collect();
    }
}
```

## Filterable Fields Per Panel

### ConfigModel
- `name` (primary, free-text)

### DagModel
- `dag_id` (primary, free-text)
- `is_paused` (enum: true, false)
- `owners` (free-text)
- `tags` (free-text)

### DagRunModel
- `dag_run_id` (primary, free-text)
- `state` (enum: running, success, failed, queued, up_for_retry)
- `run_type` (enum: scheduled, manual, backfill, dataset_triggered)

### TaskInstanceModel
- `task_id` (primary, free-text)
- `state` (enum: running, success, failed, queued, up_for_retry, skipped, etc.)

## Dependencies

No additional dependencies required - uses manual trait implementations.
