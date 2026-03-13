/// Generates a newtype wrapper around `String` with common trait implementations.
///
/// Each generated type provides:
/// - `Deref<Target = str>` for transparent `&str` access (e.g. passing to API functions)
/// - `From<String>`, `From<&str>` for ergonomic construction
/// - `Display`, `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`, `Default` via standard derives
/// - `Serialize`/`Deserialize` via `#[serde(transparent)]` so JSON fields stay as plain strings
/// - `AsRef<str>` for generic string conversion
macro_rules! define_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(
            Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord,
            serde::Serialize, serde::Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl std::ops::Deref for $name {
            type Target = str;
            fn deref(&self) -> &str {
                &self.0
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_owned())
            }
        }

        impl From<$name> for String {
            fn from(id: $name) -> Self {
                id.0
            }
        }

        impl std::borrow::Borrow<str> for $name {
            fn borrow(&self) -> &str {
                &self.0
            }
        }

        impl PartialEq<&str> for $name {
            fn eq(&self, other: &&str) -> bool {
                self.0 == *other
            }
        }

        impl PartialEq<str> for $name {
            fn eq(&self, other: &str) -> bool {
                self.0 == other
            }
        }
    };
}

define_id!(
    /// Strongly-typed identifier for an Airflow DAG.
    DagId
);

define_id!(
    /// Strongly-typed identifier for an Airflow DAG run.
    DagRunId
);

define_id!(
    /// Strongly-typed identifier for an Airflow task.
    TaskId
);

define_id!(
    /// Strongly-typed identifier for an Airflow environment / server configuration.
    EnvironmentKey
);
