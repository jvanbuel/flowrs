//! Generic filterable table widget for use across Dags, `DagRuns`, and `TaskInstances`.
//!
//! This module provides a reusable `FilterableTable<T>` that encapsulates:
//! - All items from the API
//! - Filtered items with table state for UI rendering
//! - Filter state machine with autocomplete support
//! - Visual mode selection
//! - Common navigation operations

use std::ops::RangeInclusive;

use crossterm::event::{KeyCode, KeyEvent};

use super::filter::{filter_items, FilterStateMachine, Filterable};
use super::{KeyResult, StatefulTable};

/// A generic filterable table that combines data storage, filtering, and visual selection.
///
/// This widget is designed to work with any type that implements `Filterable + Clone`.
#[derive(Clone, Default)]
pub struct FilterableTable<T> {
    /// All items from the API (unfiltered)
    pub all: Vec<T>,
    /// Filtered items with table state for rendering
    pub filtered: StatefulTable<T>,
    /// Filter state machine with autocomplete
    pub filter: FilterStateMachine,
    /// Whether visual mode (multi-select) is active
    pub visual_mode: bool,
    /// Anchor index for visual selection
    pub visual_anchor: Option<usize>,
}

impl<T: Filterable + Clone> FilterableTable<T> {
    /// Creates a new empty filterable table
    pub fn new() -> Self {
        Self {
            all: Vec::new(),
            filtered: StatefulTable::new(Vec::new()),
            filter: FilterStateMachine::default(),
            visual_mode: false,
            visual_anchor: None,
        }
    }

    /// Sets the items and applies the current filter
    pub fn set_items(&mut self, items: Vec<T>) {
        self.all = items;
        self.apply_filter();
    }

    /// Applies the current filter conditions to the items
    pub fn apply_filter(&mut self) {
        let conditions = self.filter.active_conditions();
        let filtered = filter_items(&self.all, &conditions);
        self.filtered.items = filtered;
    }

    /// Activates filter mode
    pub fn activate_filter(&mut self) {
        self.filter.activate();
        self.apply_filter();
    }

    /// Handles a key event for the filter.
    /// This handles both:
    /// - `/` key when filter is inactive (activates filter)
    /// - All filter input keys when filter is active
    pub fn handle_filter_key(&mut self, key_event: &KeyEvent) -> KeyResult {
        if self.filter.update(key_event, &T::filterable_fields()) {
            self.apply_filter();
            KeyResult::Consumed
        } else {
            KeyResult::Ignored
        }
    }

    /// Returns the currently selected item
    pub fn current(&self) -> Option<&T> {
        self.filtered
            .state
            .selected()
            .and_then(|i| self.filtered.items.get(i))
    }

    /// Returns the currently selected item mutably
    pub fn current_mut(&mut self) -> Option<&mut T> {
        self.filtered
            .state
            .selected()
            .and_then(|i| self.filtered.items.get_mut(i))
    }

    /// Returns the inclusive range of selected indices, if in visual mode
    pub fn visual_selection(&self) -> Option<RangeInclusive<usize>> {
        if !self.visual_mode {
            return None;
        }
        let anchor = self.visual_anchor?;
        let cursor = self.filtered.state.selected()?;
        let (start, end) = if anchor <= cursor {
            (anchor, cursor)
        } else {
            (cursor, anchor)
        };
        Some(start..=end)
    }

    /// Returns count of selected items (for bottom border display)
    pub fn visual_selection_count(&self) -> usize {
        self.visual_selection()
            .map_or(0, |r| r.end() - r.start() + 1)
    }

    /// Returns selected item IDs based on a provided key extractor
    /// In visual mode, returns all selected items; otherwise just the current item
    pub fn selected_ids<F>(&self, key_fn: F) -> Vec<String>
    where
        F: Fn(&T) -> String,
    {
        match self.visual_selection() {
            Some(range) => range
                .filter_map(|i| self.filtered.items.get(i))
                .map(&key_fn)
                .collect(),
            None => self
                .filtered
                .state
                .selected()
                .and_then(|i| self.filtered.items.get(i))
                .map(|item| vec![key_fn(item)])
                .unwrap_or_default(),
        }
    }

    /// Handle common navigation keys (j/k/G/gg pattern)
    pub fn handle_navigation(
        &mut self,
        key_code: KeyCode,
        event_buffer: &mut Vec<KeyCode>,
    ) -> KeyResult {
        match key_code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.filtered.next();
                KeyResult::Consumed
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.filtered.previous();
                KeyResult::Consumed
            }
            KeyCode::Char('G') => {
                if !self.filtered.items.is_empty() {
                    self.filtered
                        .state
                        .select(Some(self.filtered.items.len() - 1));
                }
                KeyResult::Consumed
            }
            KeyCode::Char('g') => {
                if let Some(last_key) = event_buffer.pop() {
                    if last_key == KeyCode::Char('g') {
                        self.filtered.state.select_first();
                    } else {
                        event_buffer.push(last_key);
                        event_buffer.push(key_code);
                    }
                } else {
                    event_buffer.push(key_code);
                }
                KeyResult::Consumed
            }
            _ => KeyResult::Ignored,
        }
    }

    /// Handle visual mode keys (V to enter, Esc to exit)
    pub fn handle_visual_mode_key(&mut self, key_code: KeyCode) -> KeyResult {
        match key_code {
            KeyCode::Char('V') => {
                if let Some(cursor) = self.filtered.state.selected() {
                    self.visual_mode = true;
                    self.visual_anchor = Some(cursor);
                }
                KeyResult::Consumed
            }
            KeyCode::Esc => {
                if self.visual_mode {
                    self.visual_mode = false;
                    self.visual_anchor = None;
                    KeyResult::Consumed
                } else {
                    KeyResult::PassThrough
                }
            }
            _ => KeyResult::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::model::filter::FilterableField;

    #[derive(Clone, Default, Debug)]
    struct TestItem {
        id: String,
        status: String,
    }

    impl Filterable for TestItem {
        fn filterable_fields() -> Vec<FilterableField> {
            vec![
                FilterableField::primary("id"),
                FilterableField::enumerated("status", vec!["running", "success", "failed"]),
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
    fn test_new_table_is_empty() {
        let table: FilterableTable<TestItem> = FilterableTable::new();
        assert!(table.filtered.items.is_empty());
        assert_eq!(table.filtered.items.len(), 0);
        assert!(table.current().is_none());
    }

    #[test]
    fn test_set_items() {
        let mut table: FilterableTable<TestItem> = FilterableTable::new();
        let items = vec![
            TestItem {
                id: "item_1".to_string(),
                status: "running".to_string(),
            },
            TestItem {
                id: "item_2".to_string(),
                status: "success".to_string(),
            },
        ];
        table.set_items(items);
        assert_eq!(table.filtered.items.len(), 2);
        assert!(!table.filtered.items.is_empty());
    }

    #[test]
    fn test_navigation() {
        let mut table: FilterableTable<TestItem> = FilterableTable::new();
        table.set_items(vec![
            TestItem {
                id: "1".to_string(),
                status: "running".to_string(),
            },
            TestItem {
                id: "2".to_string(),
                status: "success".to_string(),
            },
            TestItem {
                id: "3".to_string(),
                status: "failed".to_string(),
            },
        ]);

        // Start with no selection
        assert!(table.current().is_none());

        // Move next, should select first
        table.filtered.next();
        assert_eq!(table.current().map(|i| i.id.as_str()), Some("1"));

        // Move next
        table.filtered.next();
        assert_eq!(table.current().map(|i| i.id.as_str()), Some("2"));

        // Select last
        table
            .filtered
            .state
            .select(Some(table.filtered.items.len() - 1));
        assert_eq!(table.current().map(|i| i.id.as_str()), Some("3"));

        // Select first
        table.filtered.state.select_first();
        assert_eq!(table.current().map(|i| i.id.as_str()), Some("1"));
    }

    #[test]
    fn test_visual_mode() {
        let mut table: FilterableTable<TestItem> = FilterableTable::new();
        table.set_items(vec![
            TestItem {
                id: "1".to_string(),
                status: "running".to_string(),
            },
            TestItem {
                id: "2".to_string(),
                status: "success".to_string(),
            },
            TestItem {
                id: "3".to_string(),
                status: "failed".to_string(),
            },
        ]);

        // Select first item
        table.filtered.next();
        assert!(table.visual_selection().is_none());

        // Enter visual mode
        table.visual_mode = true;
        table.visual_anchor = table.filtered.state.selected();
        assert!(table.visual_mode);
        assert_eq!(table.visual_anchor, Some(0));
        assert_eq!(table.visual_selection_count(), 1);

        // Move down to expand selection
        table.filtered.next();
        table.filtered.next();
        assert_eq!(table.visual_selection_count(), 3);

        // Get selected IDs
        let ids = table.selected_ids(|item| item.id.clone());
        assert_eq!(ids, vec!["1", "2", "3"]);

        // Exit visual mode
        table.visual_mode = false;
        table.visual_anchor = None;
        assert!(!table.visual_mode);
        assert!(table.visual_selection().is_none());
    }

    #[test]
    fn test_selected_ids_normal_mode() {
        let mut table: FilterableTable<TestItem> = FilterableTable::new();
        table.set_items(vec![
            TestItem {
                id: "1".to_string(),
                status: "running".to_string(),
            },
            TestItem {
                id: "2".to_string(),
                status: "success".to_string(),
            },
        ]);

        // No selection
        assert!(table.selected_ids(|item| item.id.clone()).is_empty());

        // Select first item
        table.filtered.next();
        let ids = table.selected_ids(|item| item.id.clone());
        assert_eq!(ids, vec!["1"]);
    }

    #[test]
    fn test_handle_navigation() {
        let mut table: FilterableTable<TestItem> = FilterableTable::new();
        table.set_items(vec![
            TestItem {
                id: "1".to_string(),
                status: "running".to_string(),
            },
            TestItem {
                id: "2".to_string(),
                status: "success".to_string(),
            },
        ]);

        let mut buffer = Vec::new();

        // j key
        assert!(matches!(
            table.handle_navigation(KeyCode::Char('j'), &mut buffer),
            KeyResult::Consumed
        ));
        assert_eq!(table.current().map(|i| i.id.as_str()), Some("1"));

        // k key
        table.handle_navigation(KeyCode::Char('j'), &mut buffer);
        assert!(matches!(
            table.handle_navigation(KeyCode::Char('k'), &mut buffer),
            KeyResult::Consumed
        ));
        assert_eq!(table.current().map(|i| i.id.as_str()), Some("1"));

        // G key (go to last)
        assert!(matches!(
            table.handle_navigation(KeyCode::Char('G'), &mut buffer),
            KeyResult::Consumed
        ));
        assert_eq!(table.current().map(|i| i.id.as_str()), Some("2"));

        // gg (go to first) - first g
        table.handle_navigation(KeyCode::Char('g'), &mut buffer);
        assert_eq!(buffer, vec![KeyCode::Char('g')]);

        // gg - second g
        table.handle_navigation(KeyCode::Char('g'), &mut buffer);
        assert!(buffer.is_empty());
        assert_eq!(table.current().map(|i| i.id.as_str()), Some("1"));

        // Unknown key
        assert!(matches!(
            table.handle_navigation(KeyCode::Char('x'), &mut buffer),
            KeyResult::Ignored
        ));
    }

    #[test]
    fn test_handle_visual_mode_key() {
        let mut table: FilterableTable<TestItem> = FilterableTable::new();
        table.set_items(vec![
            TestItem {
                id: "1".to_string(),
                status: "running".to_string(),
            },
            TestItem {
                id: "2".to_string(),
                status: "success".to_string(),
            },
        ]);

        // Select first item
        table.filtered.next();
        assert!(!table.visual_mode);

        // V key enters visual mode
        assert!(matches!(
            table.handle_visual_mode_key(KeyCode::Char('V')),
            KeyResult::Consumed
        ));
        assert!(table.visual_mode);

        // Esc key exits visual mode when in visual mode
        assert!(matches!(
            table.handle_visual_mode_key(KeyCode::Esc),
            KeyResult::Consumed
        ));
        assert!(!table.visual_mode);

        // Esc key returns PassThrough when not in visual mode
        assert!(matches!(
            table.handle_visual_mode_key(KeyCode::Esc),
            KeyResult::PassThrough
        ));
        assert!(!table.visual_mode);

        // Unknown key returns Ignored
        assert!(matches!(
            table.handle_visual_mode_key(KeyCode::Char('x')),
            KeyResult::Ignored
        ));
    }
}
