use serde::{Deserialize, Serialize};

/// Common Log model used by the application
#[derive(Debug, Serialize, Deserialize)]
pub struct Log {
    pub continuation_token: Option<String>,
    pub content: String,
}

// From trait implementations for v1 models
impl From<crate::airflow::model::v1::log::Log> for Log {
    /// Creates a common `Log` from a v1 Airflow `log::Log` by copying its fields.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::airflow::model::v1::log as v1_log;
    /// use crate::airflow::model::common::log::Log as CommonLog;
    ///
    /// let v1 = v1_log::Log { continuation_token: Some("token".into()), content: "line".into() };
    /// let common: CommonLog = v1.into();
    /// assert_eq!(common.continuation_token.as_deref(), Some("token"));
    /// assert_eq!(common.content, "line");
    /// ```
    fn from(value: crate::airflow::model::v1::log::Log) -> Self {
        Log {
            continuation_token: value.continuation_token,
            content: value.content,
        }
    }
}

// From trait implementations for v2 models
impl From<crate::airflow::model::v2::log::Log> for Log {
    /// Converts a v2 Airflow log model into the common `Log` model.
    ///
    /// # Examples
    ///
    /// ```
    /// let v2 = crate::airflow::model::v2::log::Log {
    ///     continuation_token: Some("token".to_string()),
    ///     content: "line".to_string(),
    /// };
    /// let common: crate::airflow::model::common::log::Log = v2.into();
    /// assert_eq!(common.content, "line");
    /// assert_eq!(common.continuation_token.as_deref(), Some("token"));
    /// ```
    fn from(value: crate::airflow::model::v2::log::Log) -> Self {
        Log {
            continuation_token: value.continuation_token,
            content: value.content,
        }
    }
}