use ratatui::widgets::TableState;

use super::{events::custom::FlowrsEvent, worker::WorkerMessage};

pub mod config;
pub mod dagruns;
pub mod dags;
pub mod filter;
pub mod logs;
pub mod popup;
pub mod taskinstances;

pub trait Model {
    fn update(&mut self, event: &FlowrsEvent) -> (Option<FlowrsEvent>, Vec<WorkerMessage>);
}

#[derive(Clone)]
pub struct StatefulTable<T> {
    pub state: TableState,
    pub items: Vec<T>,
}

impl<T> StatefulTable<T> {
    pub fn new(items: Vec<T>) -> StatefulTable<T> {
        StatefulTable {
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
                    i.saturating_sub(1)
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
    fn test_next_with_empty_items() {
        let mut table: StatefulTable<i32> = StatefulTable::new(vec![]);
        
        // Should not panic and state should remain None
        table.next();
        assert_eq!(table.state.selected(), None);
        
        // Try multiple times to ensure stability
        table.next();
        table.next();
        assert_eq!(table.state.selected(), None);
    }

    #[test]
    fn test_previous_with_empty_items() {
        let mut table: StatefulTable<i32> = StatefulTable::new(vec![]);
        
        // Should not panic and state should remain None
        table.previous();
        assert_eq!(table.state.selected(), None);
        
        // Try multiple times to ensure stability
        table.previous();
        table.previous();
        assert_eq!(table.state.selected(), None);
    }

    #[test]
    fn test_next_with_single_item() {
        let mut table = StatefulTable::new(vec![1]);
        
        // First call should select index 0
        table.next();
        assert_eq!(table.state.selected(), Some(0));
        
        // Next call should wrap to 0
        table.next();
        assert_eq!(table.state.selected(), Some(0));
        
        // Another call should still be 0
        table.next();
        assert_eq!(table.state.selected(), Some(0));
    }

    #[test]
    fn test_previous_with_single_item() {
        let mut table = StatefulTable::new(vec![1]);
        
        // First call should select index 0
        table.previous();
        assert_eq!(table.state.selected(), Some(0));
        
        // Previous call should stay at 0
        table.previous();
        assert_eq!(table.state.selected(), Some(0));
        
        // Another call should still be 0
        table.previous();
        assert_eq!(table.state.selected(), Some(0));
    }

    #[test]
    fn test_next_wrapping() {
        let mut table = StatefulTable::new(vec![1, 2, 3]);
        
        // Start at None, first next goes to 0
        table.next();
        assert_eq!(table.state.selected(), Some(0));
        
        // Move through items
        table.next();
        assert_eq!(table.state.selected(), Some(1));
        
        table.next();
        assert_eq!(table.state.selected(), Some(2));
        
        // Should wrap back to 0
        table.next();
        assert_eq!(table.state.selected(), Some(0));
    }

    #[test]
    fn test_previous_wrapping() {
        let mut table = StatefulTable::new(vec![1, 2, 3]);
        
        // Start at None, first previous goes to 0
        table.previous();
        assert_eq!(table.state.selected(), Some(0));
        
        // Should wrap to last item (2)
        table.previous();
        assert_eq!(table.state.selected(), Some(2));
        
        // Move backwards
        table.previous();
        assert_eq!(table.state.selected(), Some(1));
        
        table.previous();
        assert_eq!(table.state.selected(), Some(0));
        
        // Wrap again
        table.previous();
        assert_eq!(table.state.selected(), Some(2));
    }

    #[test]
    fn test_navigation_with_preselected_index() {
        let mut table = StatefulTable::new(vec![1, 2, 3, 4, 5]);
        
        // Manually select middle item
        table.state.select(Some(2));
        
        // Next should go to 3
        table.next();
        assert_eq!(table.state.selected(), Some(3));
        
        // Previous should go to 2
        table.previous();
        assert_eq!(table.state.selected(), Some(2));
        
        // Previous should go to 1
        table.previous();
        assert_eq!(table.state.selected(), Some(1));
    }

    #[test]
    fn test_saturating_sub_behavior() {
        let mut table = StatefulTable::new(vec![1, 2]);
        
        // Manually set selection to 0
        table.state.select(Some(0));
        
        // Previous from 0 should wrap to last (using saturating_sub)
        table.previous();
        assert_eq!(table.state.selected(), Some(1));
        
        // Next from last should wrap to first
        table.next();
        assert_eq!(table.state.selected(), Some(0));
    }
}
