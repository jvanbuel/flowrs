/// A single filter condition (e.g., "state contains 'running'")
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilterCondition {
    /// The field name to filter on (e.g., "state", "`dag_run_id`")
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
