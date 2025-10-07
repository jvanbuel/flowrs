use serde::{Deserialize, Serialize};

/// Common Log model used by the application
#[derive(Debug, Serialize, Deserialize)]
pub struct Log {
    pub continuation_token: Option<String>,
    pub content: String,
}

// From trait implementations for v1 models
impl From<crate::airflow::model::v1::log::Log> for Log {
    fn from(value: crate::airflow::model::v1::log::Log) -> Self {
        Log {
            continuation_token: value.continuation_token,
            content: value.content,
        }
    }
}

// From trait implementations for v2 models
impl From<crate::airflow::model::v2::log::Log> for Log {
    fn from(value: crate::airflow::model::v2::log::Log) -> Self {
        Log {
            continuation_token: value.continuation_token,
            content: value.content,
        }
    }
}
