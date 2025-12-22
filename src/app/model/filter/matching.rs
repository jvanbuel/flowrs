use super::{FilterCondition, Filterable};

/// Check if an item matches all filter conditions
fn item_matches<T: Filterable>(item: &T, conditions: &[FilterCondition]) -> bool {
    conditions.iter().all(|cond| {
        let field_name = if cond.is_primary {
            T::primary_field()
        } else {
            &cond.field
        };

        item.get_field_value(field_name)
            .as_ref()
            .is_some_and(|v| cond.matches(v))
    })
}

/// Check if an item matches all filter conditions (public API, used by tests)
#[allow(dead_code)]
pub fn matches<T: Filterable>(item: &T, conditions: &[FilterCondition]) -> bool {
    item_matches(item, conditions)
}

/// Filter a collection of items by conditions
pub fn filter_items<T: Filterable + Clone>(items: &[T], conditions: &[FilterCondition]) -> Vec<T> {
    if conditions.is_empty() {
        return items.to_vec();
    }

    items
        .iter()
        .filter(|item| item_matches(*item, conditions))
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
                FilterableField::enumerated("status", vec!["running", "failed", "success"]),
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
            status: "running".to_string(),
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
            status: "running".to_string(),
        };

        let cond = FilterCondition::new("status", "run", false);
        assert!(matches(&item, &[cond]));

        let cond = FilterCondition::new("status", "failed", false);
        assert!(!matches(&item, &[cond]));
    }

    #[test]
    fn test_matches_multiple_conditions() {
        let item = TestItem {
            id: "my_item".to_string(),
            status: "running".to_string(),
        };

        let conditions = vec![
            FilterCondition::primary("my"),
            FilterCondition::new("status", "running", false),
        ];
        assert!(matches(&item, &conditions));

        let conditions = vec![
            FilterCondition::primary("my"),
            FilterCondition::new("status", "failed", false),
        ];
        assert!(!matches(&item, &conditions)); // Second doesn't match
    }

    #[test]
    fn test_filter_items() {
        let items = vec![
            TestItem {
                id: "item_1".to_string(),
                status: "running".to_string(),
            },
            TestItem {
                id: "item_2".to_string(),
                status: "failed".to_string(),
            },
            TestItem {
                id: "other_3".to_string(),
                status: "running".to_string(),
            },
        ];

        // Filter by primary
        let conditions = vec![FilterCondition::primary("item")];
        let filtered = filter_items(&items, &conditions);
        assert_eq!(filtered.len(), 2);

        // Filter by status
        let conditions = vec![FilterCondition::new("status", "running", false)];
        let filtered = filter_items(&items, &conditions);
        assert_eq!(filtered.len(), 2);

        // Combined
        let conditions = vec![
            FilterCondition::primary("item"),
            FilterCondition::new("status", "running", false),
        ];
        let filtered = filter_items(&items, &conditions);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "item_1");
    }

    #[test]
    fn test_empty_conditions() {
        let items = vec![TestItem {
            id: "item_1".to_string(),
            status: "running".to_string(),
        }];

        let filtered = filter_items(&items, &[]);
        assert_eq!(filtered.len(), 1);
    }
}
