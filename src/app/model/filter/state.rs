use super::{AutocompleteState, FilterCondition, FilterKind};

/// The filter state machine states
#[derive(Clone, Debug, Default)]
pub enum FilterState {
    /// Filter UI is hidden
    #[default]
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

impl FilterState {
    /// Check if the filter is currently active (visible)
    pub const fn is_active(&self) -> bool {
        !matches!(self, Self::Inactive)
    }

    /// Get all currently active filter conditions (including in-progress typing).
    /// The `primary_field` is used to create conditions for Default mode filtering.
    pub fn active_conditions(&self, primary_field: Option<&str>) -> Vec<FilterCondition> {
        match self {
            Self::Inactive => vec![],

            Self::Default {
                autocomplete,
                conditions,
            } => {
                let mut result = conditions.clone();
                if !autocomplete.typed.is_empty() {
                    if let Some(field) = primary_field {
                        result.push(FilterCondition::new(field, &autocomplete.typed, false));
                    } else {
                        // Fallback to legacy "primary" field if not set
                        result.push(FilterCondition::primary(&autocomplete.typed));
                    }
                }
                result
            }

            Self::AttributeSelection { conditions, .. } => {
                // While selecting attribute, only apply confirmed conditions
                conditions.clone()
            }

            Self::ValueInput {
                field,
                autocomplete,
                conditions,
                ..
            } => {
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
    pub const fn autocomplete(&self) -> Option<&AutocompleteState> {
        match self {
            Self::Inactive => None,
            Self::Default { autocomplete, .. }
            | Self::AttributeSelection { autocomplete, .. }
            | Self::ValueInput { autocomplete, .. } => Some(autocomplete),
        }
    }

    /// Get mutable reference to autocomplete state
    pub const fn autocomplete_mut(&mut self) -> Option<&mut AutocompleteState> {
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
            autocomplete: AutocompleteState::with_typed("my_dag", vec![]),
            conditions: vec![],
        };

        // With primary field set
        let conditions = state.active_conditions(Some("dag_id"));
        assert_eq!(conditions.len(), 1);
        assert_eq!(conditions[0].field, "dag_id");
        assert_eq!(conditions[0].value, "my_dag");

        // Without primary field (legacy behavior)
        let conditions = state.active_conditions(None);
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
            autocomplete: AutocompleteState::with_typed("my", vec![]),
            conditions: vec![confirmed.clone()],
        };

        let conditions = state.active_conditions(None);
        assert_eq!(conditions.len(), 2);
        assert_eq!(conditions[0], confirmed);
        assert_eq!(conditions[1].field, "dag_id");
        assert_eq!(conditions[1].value, "my");
    }

    #[test]
    fn test_attribute_selection_only_confirmed() {
        let confirmed = FilterCondition::new("state", "running", false);
        let state = FilterState::AttributeSelection {
            autocomplete: AutocompleteState::with_typed("dag", vec!["dag_id".to_string()]),
            conditions: vec![confirmed.clone()],
        };

        let conditions = state.active_conditions(None);
        assert_eq!(conditions.len(), 1);
        assert_eq!(conditions[0], confirmed);
    }
}
