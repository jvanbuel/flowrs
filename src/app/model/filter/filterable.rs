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

/// Macro to implement `Filterable` for a type with compile-time field validation.
///
/// This macro eliminates stringly-typed field matching by generating both
/// `filterable_fields()` and `get_field_value()` from a single definition.
///
/// # Example
/// ```ignore
/// impl_filterable! {
///     Dag,
///     primary: dag_id => |s| Some(s.dag_id.clone()),
///     fields: [
///         is_paused: enum["true", "false"] => |s| Some(s.is_paused.to_string()),
///         owners => |s| Some(s.owners.join(", ")),
///     ]
/// }
/// ```
#[macro_export]
macro_rules! impl_filterable {
    (
        $type:ty,
        primary: $primary_field:ident => $primary_accessor:expr,
        fields: [
            $( $field_name:ident $(: enum[$($variant:literal),+ $(,)?])? => $accessor:expr ),* $(,)?
        ]
    ) => {
        impl $crate::app::model::filter::Filterable for $type {
            fn filterable_fields() -> Vec<$crate::app::model::filter::FilterableField> {
                vec![
                    $crate::app::model::filter::FilterableField::primary(stringify!($primary_field)),
                    $(
                        impl_filterable!(@field $field_name $(: enum[$($variant),+])?),
                    )*
                ]
            }

            fn get_field_value(&self, field_name: &str) -> Option<String> {
                match field_name {
                    stringify!($primary_field) => ($primary_accessor)(self),
                    $(
                        stringify!($field_name) => ($accessor)(self),
                    )*
                    _ => None,
                }
            }
        }
    };

    // Helper: enumerated field
    (@field $field_name:ident : enum[$($variant:literal),+ $(,)?]) => {
        $crate::app::model::filter::FilterableField::enumerated(
            stringify!($field_name),
            vec![$($variant),+]
        )
    };

    // Helper: free-text field (no enum specified)
    (@field $field_name:ident) => {
        $crate::app::model::filter::FilterableField::free_text(stringify!($field_name))
    };
}

/// Trait for types that can be filtered in the TUI
pub trait Filterable {
    /// Returns all filterable fields with their metadata.
    /// The first field marked as primary (via `FilterableField::primary()`) is used as the default.
    fn filterable_fields() -> Vec<FilterableField>;

    /// Get the value of a field by name for filtering
    fn get_field_value(&self, field_name: &str) -> Option<String>;

    /// Returns the name of the primary filter field.
    /// Default implementation finds the first field with `is_primary: true`.
    fn primary_field() -> &'static str {
        Self::filterable_fields()
            .iter()
            .find(|f| f.is_primary)
            .map(|f| f.name)
            .expect("Filterable type must have a primary field")
    }
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
