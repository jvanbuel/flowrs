use ratatui::widgets::TableState;

use super::{events::custom::FlowrsEvent, state::NavigationContext, worker::WorkerMessage};

pub mod config;
pub mod dagruns;
pub mod dags;
pub mod filter;
pub mod filterable_table;
pub mod logs;
pub mod popup;
pub mod taskinstances;

pub use filterable_table::FilterableTable;
pub use popup::Popup;

/// Result of handling a key event in the chain of responsibility pattern.
///
/// Handlers return this to indicate whether they consumed the event and
/// what messages/events should be produced.
#[derive(Debug)]
pub enum KeyResult {
    /// Event was consumed, no further processing needed
    Consumed,
    /// Event was consumed and produced worker messages
    ConsumedWith(Vec<WorkerMessage>),
    /// Event was consumed but should pass through (for panel navigation)
    PassThrough,
    /// Event was consumed, pass through with messages
    PassWith(Vec<WorkerMessage>),
    /// Event was not handled by this handler, try next in chain
    Ignored,
}

impl KeyResult {
    /// Chain handlers: if this result is Ignored, try the next handler
    #[must_use]
    pub fn or_else<F: FnOnce() -> KeyResult>(self, f: F) -> KeyResult {
        match self {
            KeyResult::Ignored => f(),
            other => other,
        }
    }

    /// Convert to the `update()` return type
    #[allow(clippy::match_same_arms)] // PassThrough and Ignored are semantically different
    pub fn into_result(self, event: &FlowrsEvent) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        match self {
            KeyResult::Consumed => (None, vec![]),
            KeyResult::ConsumedWith(msgs) => (None, msgs),
            KeyResult::PassThrough => (Some(event.clone()), vec![]),
            KeyResult::PassWith(msgs) => (Some(event.clone()), msgs),
            KeyResult::Ignored => (Some(event.clone()), vec![]),
        }
    }

    /// Create from a simple bool (true = consumed, false = ignored)
    pub fn from_consumed(consumed: bool) -> Self {
        if consumed {
            KeyResult::Consumed
        } else {
            KeyResult::Ignored
        }
    }
}

pub trait Model {
    fn update(
        &mut self,
        event: &FlowrsEvent,
        ctx: &NavigationContext,
    ) -> (Option<FlowrsEvent>, Vec<WorkerMessage>);
}

#[derive(Clone, Default)]
pub struct StatefulTable<T> {
    pub state: TableState,
    pub items: Vec<T>,
}

impl<T> StatefulTable<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self {
            state: TableState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_empty() {
        let mut table: StatefulTable<i32> = StatefulTable::new(vec![]);
        table.next();
        table.previous();
        assert_eq!(table.state.selected(), None);
    }

    #[test]
    fn test_navigation_wrapping() {
        // Single item: always stays at 0
        let mut table = StatefulTable::new(vec![1]);
        table.next();
        assert_eq!(table.state.selected(), Some(0));
        table.next();
        assert_eq!(table.state.selected(), Some(0));
        table.previous();
        assert_eq!(table.state.selected(), Some(0));

        // Multiple items: wraps forward past end and backward past start
        let mut table = StatefulTable::new(vec![1, 2, 3]);
        table.next(); // None → 0
        table.next(); // 0 → 1
        table.next(); // 1 → 2
        table.next(); // 2 → 0 (wrap)
        assert_eq!(table.state.selected(), Some(0));
        table.previous(); // 0 → 2 (wrap)
        assert_eq!(table.state.selected(), Some(2));
    }
}
