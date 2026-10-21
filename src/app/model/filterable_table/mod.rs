//! Generic filterable table widget for use across Dags, `DagRuns`, and `TaskInstances`.
//!
//! This module provides a reusable `FilterableTable<T>` that encapsulates:
//! - All items from the API
//! - Filtered items with table state for UI rendering
//! - Filter state machine with autocomplete support
//! - Visual mode selection
//! - Common navigation operations

mod render;

use std::ops::RangeInclusive;
use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::TableState;

use super::filter::{filter_items, FilterStateMachine, Filterable};
use super::{KeyResult, StatefulTable};

/// A generic filterable, selectable table shared by the Dags, `DagRuns`, and
/// `TaskInstances` panels.
///
/// It shares the canonical `all` items with the environment cache, which
/// keeps them sorted, and derives a filtered *view* of indices into them, so
/// items are never cloned. Callers interact through the item-oriented API
/// (`items`, `current`, ...) and never see the index representation.
#[derive(Debug, Clone)]
pub struct FilterableTable<T> {
    /// Canonical items, in display order; shared with the environment cache.
    all: Arc<[T]>,
    /// Indices into `all` that pass the filter, with the table selection state.
    view: StatefulTable<usize>,
    /// Filter state machine with autocomplete.
    filter: FilterStateMachine,
    /// Anchor position for visual selection; `Some` means visual mode is active.
    visual_anchor: Option<usize>,
    /// Put the cursor on the first row once the view has rows; set when the
    /// table's content is about to belong to a different selection.
    select_first_row: bool,
}

impl<T> Default for FilterableTable<T> {
    fn default() -> Self {
        Self {
            all: Arc::default(),
            view: StatefulTable::default(),
            filter: FilterStateMachine::default(),
            visual_anchor: None,
            select_first_row: false,
        }
    }
}

impl<T: Filterable> FilterableTable<T> {
    /// Creates a new empty filterable table
    pub fn new() -> Self {
        Self {
            all: Arc::default(),
            view: StatefulTable::new(Vec::new()),
            filter: FilterStateMachine::default(),
            visual_anchor: None,
            select_first_row: false,
        }
    }

    // ── Data ────────────────────────────────────────────────

    /// Replace all items and recompute the filtered view. The cursor stays
    /// on the row it was on, wherever that row now is; if that row is gone,
    /// it keeps its position, clamped to the last row.
    pub fn set_items(&mut self, items: impl Into<Arc<[T]>>) {
        let current_key = self.current().map(Filterable::row_key);
        self.all = items.into();
        self.apply_filter();
        let followed =
            current_key.and_then(|key| self.items().position(|item| item.row_key() == key));
        let clamped = self
            .view
            .state
            .selected()
            .map(|position| position.min(self.len().saturating_sub(1)));
        if let Some(position) = followed.or(clamped) {
            self.view
                .state
                .select((!self.is_empty()).then_some(position));
        }
    }

    /// Remove all items, along with the selection and any visual-mode anchor
    /// that pointed into them. The filter text is kept.
    pub fn clear(&mut self) {
        self.all = Arc::default();
        self.view = StatefulTable::new(Vec::new());
        self.visual_anchor = None;
    }

    /// Read-only access to the canonical items.
    pub fn all(&self) -> &[T] {
        &self.all
    }

    /// Recompute the filtered view from the current filter conditions.
    pub fn apply_filter(&mut self) {
        let conditions = self.filter.active_conditions();
        self.view.items = filter_items(&self.all, &conditions);
        if self.select_first_row && !self.view.items.is_empty() {
            self.view.state.select(Some(0));
            self.select_first_row = false;
        }
    }

    /// Forget the cursor and visual selection; the cursor goes to the first
    /// row as soon as the table has rows.
    pub fn reset_cursor(&mut self) {
        self.view.state.select(None);
        self.visual_anchor = None;
        self.select_first_row = true;
    }

    // ── The filtered view ───────────────────────────────────

    /// Iterate the filtered items in display order.
    pub fn items(&self) -> impl Iterator<Item = &T> {
        self.view.items.iter().filter_map(|&i| self.all.get(i))
    }

    /// Number of items in the filtered view.
    pub fn len(&self) -> usize {
        self.view.items.len()
    }

    /// Whether the filtered view is empty.
    pub fn is_empty(&self) -> bool {
        self.view.items.is_empty()
    }

    /// Resolve a position in the filtered view to its item.
    pub fn item_at(&self, pos: usize) -> Option<&T> {
        self.all.get(*self.view.items.get(pos)?)
    }

    /// The currently selected item, if any.
    pub fn current(&self) -> Option<&T> {
        self.item_at(self.view.state.selected()?)
    }

    /// The selected position within the filtered view.
    pub fn selected_position(&self) -> Option<usize> {
        self.view.state.selected()
    }

    /// Mutable table state, for driving the ratatui `StatefulWidget` render.
    pub fn state_mut(&mut self) -> &mut TableState {
        &mut self.view.state
    }

    // ── Filter ──────────────────────────────────────────────

    /// Read-only access to the filter state (active flag, cursor, display).
    pub fn filter(&self) -> &FilterStateMachine {
        &self.filter
    }

    /// Mutable access to the filter state (e.g. to set autocomplete values).
    pub fn filter_mut(&mut self) -> &mut FilterStateMachine {
        &mut self.filter
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

    // ── Selection / visual mode ─────────────────────────────

    /// Returns the inclusive range of selected positions, if in visual mode
    pub fn visual_selection(&self) -> Option<RangeInclusive<usize>> {
        let anchor = self.visual_anchor?;
        let cursor = self.view.state.selected()?;
        Some(anchor.min(cursor)..=anchor.max(cursor))
    }

    /// Exit visual selection mode.
    pub fn clear_visual_mode(&mut self) {
        self.visual_anchor = None;
    }

    /// Returns count of selected items (for bottom border display)
    pub fn visual_selection_count(&self) -> usize {
        self.visual_selection()
            .map_or(0, |r| r.end() - r.start() + 1)
    }

    /// Returns selected item IDs based on a provided key extractor
    /// In visual mode, returns all selected items; otherwise just the current item
    pub fn selected_ids<F, R>(&self, key_fn: F) -> Vec<R>
    where
        F: Fn(&T) -> R,
    {
        match self.visual_selection() {
            Some(range) => range
                .filter_map(|pos| self.item_at(pos))
                .map(&key_fn)
                .collect(),
            None => self
                .current()
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
                self.view.next();
                KeyResult::Consumed
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.view.previous();
                KeyResult::Consumed
            }
            KeyCode::Char('G') => {
                if !self.view.items.is_empty() {
                    self.view.state.select(Some(self.view.items.len() - 1));
                }
                KeyResult::Consumed
            }
            KeyCode::Char('g') => {
                if let Some(last_key) = event_buffer.pop() {
                    if last_key == KeyCode::Char('g') {
                        self.view.state.select_first();
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
                if let Some(cursor) = self.view.state.selected() {
                    self.visual_anchor = Some(cursor);
                }
                KeyResult::Consumed
            }
            KeyCode::Esc => {
                if self.visual_anchor.is_some() {
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
        assert_eq!(table.view.items.len(), 0);
        assert_eq!(table.view.items.len(), 0);
        assert!(table.current().is_none());
    }

    #[test]
    fn clear_drops_the_selection_and_visual_anchor() {
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
        table.view.next();
        table.view.next();
        table.visual_anchor = Some(0);
        assert_eq!(table.selected_position(), Some(1));

        table.clear();

        assert_eq!(table.len(), 0);
        assert!(table.current().is_none());
        assert_eq!(table.selected_position(), None);
        assert!(table.visual_anchor.is_none());
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
        assert_eq!(table.view.items.len(), 2);
        assert_ne!(table.view.items.len(), 0);
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
        table.view.next();
        assert_eq!(table.current().map(|i| i.id.as_str()), Some("1"));

        // Move next
        table.view.next();
        assert_eq!(table.current().map(|i| i.id.as_str()), Some("2"));

        // Select last
        table.view.state.select(Some(table.view.items.len() - 1));
        assert_eq!(table.current().map(|i| i.id.as_str()), Some("3"));

        // Select first
        table.view.state.select_first();
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
        table.view.next();
        assert!(table.visual_selection().is_none());

        // Enter visual mode
        table.visual_anchor = table.view.state.selected();
        assert!(table.visual_anchor.is_some());
        assert_eq!(table.visual_anchor, Some(0));
        assert_eq!(table.visual_selection_count(), 1);

        // Move down to expand selection
        table.view.next();
        table.view.next();
        assert_eq!(table.visual_selection_count(), 3);

        // Get selected IDs
        let ids = table.selected_ids(|item| item.id.clone());
        assert_eq!(ids, vec!["1", "2", "3"]);

        // Exit visual mode
        table.visual_anchor = None;
        assert!(table.visual_anchor.is_none());
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
        assert_eq!(table.selected_ids(|item| item.id.clone()).len(), 0);

        // Select first item
        table.view.next();
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
        assert_eq!(buffer.len(), 0);
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
        table.view.next();
        assert!(table.visual_anchor.is_none());

        // V key enters visual mode
        assert!(matches!(
            table.handle_visual_mode_key(KeyCode::Char('V')),
            KeyResult::Consumed
        ));
        assert!(table.visual_anchor.is_some());

        // Esc key exits visual mode when in visual mode
        assert!(matches!(
            table.handle_visual_mode_key(KeyCode::Esc),
            KeyResult::Consumed
        ));
        assert!(table.visual_anchor.is_none());

        // Esc key returns PassThrough when not in visual mode
        assert!(matches!(
            table.handle_visual_mode_key(KeyCode::Esc),
            KeyResult::PassThrough
        ));
        assert!(table.visual_anchor.is_none());

        // Unknown key returns Ignored
        assert!(matches!(
            table.handle_visual_mode_key(KeyCode::Char('x')),
            KeyResult::Ignored
        ));
    }

    fn item(id: &str) -> TestItem {
        TestItem {
            id: id.to_string(),
            status: "running".to_string(),
        }
    }

    fn current_id(table: &FilterableTable<TestItem>) -> Option<String> {
        table.current().map(|item| item.id.clone())
    }

    #[test]
    fn the_cursor_stays_on_its_row_when_a_row_is_added_above() {
        let mut table = FilterableTable::new();
        table.set_items(vec![item("b"), item("c")]);
        table.state_mut().select(Some(1));

        table.set_items(vec![item("a"), item("b"), item("c")]);

        assert_eq!(current_id(&table).as_deref(), Some("c"));
    }

    #[test]
    fn the_cursor_moves_to_the_last_row_when_its_row_at_the_end_disappears() {
        let mut table = FilterableTable::new();
        table.set_items(vec![item("a"), item("b"), item("c")]);
        table.state_mut().select(Some(2));

        table.set_items(vec![item("a"), item("b")]);

        assert_eq!(current_id(&table).as_deref(), Some("b"));
    }

    #[test]
    fn the_cursor_tells_mapped_task_instances_apart() {
        use crate::airflow::model::common::TaskInstance;
        let mapped = |map_index| TaskInstance {
            task_id: "process".into(),
            map_index,
            ..Default::default()
        };
        let mut table = FilterableTable::new();
        table.set_items(vec![mapped(0), mapped(1)]);
        table.state_mut().select(Some(1));

        table.set_items(vec![mapped(2), mapped(0), mapped(1)]);

        assert_eq!(table.current().map(|ti| ti.map_index), Some(1));
    }
}
