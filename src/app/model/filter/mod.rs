//! Filter state machine with autocomplete support
//!
//! This module provides a filter state machine that supports:
//! - Autocomplete with inline ghost text
//! - Multiple attribute filtering with AND logic
//! - Enum-aware value completion

mod autocomplete;
mod condition;
mod filterable;
mod impls;
mod matching;
mod state;
mod state_machine;
mod widget;

pub use autocomplete::AutocompleteState;
pub use condition::FilterCondition;
pub use filterable::{FilterKind, Filterable, FilterableField};
pub use matching::filter_items;
pub use state::FilterState;
pub use state_machine::FilterStateMachine;
