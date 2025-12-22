/// Describes the kind of filter values a field accepts
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilterKind {
    /// Free-text field with no predefined values
    FreeText,
    /// Field with known enum-like values for autocomplete
    Enum(Vec<&'static str>),
}

impl FilterKind {
    /// Get the available values for autocomplete (empty for `FreeText`)
    pub fn values(&self) -> Vec<String> {
        match self {
            Self::FreeText => vec![],
            Self::Enum(values) => values.iter().map(|s| (*s).to_string()).collect(),
        }
    }
}

/// Describes a field that can be filtered
#[derive(Clone, Debug)]
pub struct FilterableField {
    /// The field name as it appears in the struct
    pub name: &'static str,
    /// What kind of values this field accepts
    pub kind: FilterKind,
    /// Whether this is the primary/default filter field
    pub is_primary: bool,
}

impl FilterableField {
    pub const fn primary(name: &'static str) -> Self {
        Self {
            name,
            kind: FilterKind::FreeText,
            is_primary: true,
        }
    }

    pub const fn free_text(name: &'static str) -> Self {
        Self {
            name,
            kind: FilterKind::FreeText,
            is_primary: false,
        }
    }

    pub fn enumerated(name: &'static str, values: Vec<&'static str>) -> Self {
        Self {
            name,
            kind: FilterKind::Enum(values),
            is_primary: false,
        }
    }
}

/// Trait for types that can be filtered in the TUI
pub trait Filterable {
    /// Returns the name of the primary filter field
    fn primary_field() -> &'static str;

    /// Returns all filterable fields with their metadata
    fn filterable_fields() -> Vec<FilterableField>;

    /// Get the value of a field by name for filtering
    fn get_field_value(&self, field_name: &str) -> Option<String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_kind_values() {
        let free = FilterKind::FreeText;
        assert!(free.values().is_empty());

        let enumerated = FilterKind::Enum(vec!["running", "success", "failed"]);
        assert_eq!(enumerated.values(), vec!["running", "success", "failed"]);
    }

    #[test]
    fn test_filterable_field_constructors() {
        let primary = FilterableField::primary("dag_id");
        assert!(primary.is_primary);
        assert_eq!(primary.name, "dag_id");

        let free = FilterableField::free_text("description");
        assert!(!free.is_primary);
        assert!(matches!(free.kind, FilterKind::FreeText));

        let enumerated = FilterableField::enumerated("state", vec!["running", "success"]);
        assert!(!enumerated.is_primary);
        assert!(matches!(enumerated.kind, FilterKind::Enum(_)));
    }
}
