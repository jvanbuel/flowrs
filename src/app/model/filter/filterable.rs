use std::borrow::Cow;

/// Describes the kind of filter values a field accepts
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterKind {
    /// Free-text field with no predefined values
    FreeText,
    /// Field with known enum-like values for autocomplete
    Enum(&'static [&'static str]),
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

/// Describes a field that can be filtered.
///
/// Every constructor is `const`, so a type's field list is a static table
/// built at compile time rather than a `Vec` re-allocated on each key press.
#[derive(Clone, Copy, Debug)]
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

    pub const fn enumerated(name: &'static str, values: &'static [&'static str]) -> Self {
        Self {
            name,
            kind: FilterKind::Enum(values),
            is_primary: false,
        }
    }
}

/// Macro to implement `Filterable` for a type with compile-time field validation.
///
/// This macro eliminates stringly-typed field matching by generating
/// `filterable_fields()`, `primary_field()` and `get_field_value()` from a
/// single definition. Accessors are plain expressions over the binding named
/// after `as`, and evaluate to `Option<Cow<'_, str>>`: borrow fields that are
/// already strings, build a `String` only when the value has to be derived.
///
/// # Example
/// ```ignore
/// impl_filterable! {
///     Dag as dag,
///     primary: dag_id => Some(Cow::Borrowed(&dag.dag_id)),
///     fields: [
///         is_paused: enum["true", "false"] => Some(Cow::Borrowed(if dag.is_paused { "true" } else { "false" })),
///         owners => Some(Cow::Owned(dag.owners.join(", "))),
///     ]
/// }
/// ```
#[macro_export]
macro_rules! impl_filterable {
    (
        $type:ty as $item:ident,
        primary: $primary_field:ident => $primary_accessor:expr,
        fields: [
            $( $field_name:ident $(: enum[$($variant:literal),+ $(,)?])? => $accessor:expr ),* $(,)?
        ]
    ) => {
        impl $crate::app::model::filter::Filterable for $type {
            fn filterable_fields() -> &'static [$crate::app::model::filter::FilterableField] {
                const FIELDS: &[$crate::app::model::filter::FilterableField] = &[
                    $crate::app::model::filter::FilterableField::primary(stringify!($primary_field)),
                    $(
                        impl_filterable!(@field $field_name $(: enum[$($variant),+])?),
                    )*
                ];
                FIELDS
            }

            fn primary_field() -> &'static str {
                stringify!($primary_field)
            }

            fn get_field_value(&self, field_name: &str) -> Option<std::borrow::Cow<'_, str>> {
                let $item = self;
                match field_name {
                    stringify!($primary_field) => $primary_accessor,
                    $(
                        stringify!($field_name) => $accessor,
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
            &[$($variant),+]
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
    fn filterable_fields() -> &'static [FilterableField];

    /// Get the value of a field by name for filtering.
    ///
    /// Returns a borrow whenever the field is already text so the match loop
    /// does not allocate per item; derived values (joined lists, numbers) are
    /// returned owned.
    fn get_field_value(&self, field_name: &str) -> Option<Cow<'_, str>>;

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

        let enumerated = FilterKind::Enum(&["running", "success", "failed"]);
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

        let enumerated = FilterableField::enumerated("state", &["running", "success"]);
        assert!(!enumerated.is_primary);
        assert!(matches!(enumerated.kind, FilterKind::Enum(_)));
    }
}
