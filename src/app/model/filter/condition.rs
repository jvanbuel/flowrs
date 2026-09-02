/// A single filter condition (e.g., "state contains 'running'")
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilterCondition {
    /// The field name to filter on (e.g., "state", "`dag_run_id`")
    pub field: String,
    /// The value to match (substring match), as the user typed it
    pub value: String,
    /// Whether this is filtering on the primary/default field
    pub is_primary: bool,
    /// `value` lower-cased once at construction. Matching runs once per item
    /// per condition, so the needle is normalised here rather than in the loop.
    needle: String,
}

impl FilterCondition {
    pub fn new(field: impl Into<String>, value: impl Into<String>, is_primary: bool) -> Self {
        let value = value.into();
        Self {
            field: field.into(),
            needle: value.to_lowercase(),
            value,
            is_primary,
        }
    }

    pub fn primary(value: impl Into<String>) -> Self {
        Self::new(String::new(), value, true)
    }

    /// Check if this condition matches a field value (case-insensitive substring)
    pub fn matches(&self, field_value: &str) -> bool {
        contains_ignore_case(field_value, &self.needle)
    }
}

/// Case-insensitive substring test where `needle` is already lower-cased.
///
/// ASCII text (every Airflow identifier and state name) is compared in place
/// over the bytes; only non-ASCII haystacks pay for a lower-cased copy.
fn contains_ignore_case(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    if haystack.is_ascii() && needle.is_ascii() {
        let needle = needle.as_bytes();
        return haystack
            .as_bytes()
            .windows(needle.len())
            .any(|window| window.eq_ignore_ascii_case(needle));
    }
    haystack.to_lowercase().contains(needle)
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
