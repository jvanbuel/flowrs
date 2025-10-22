use serde::{Deserialize, Serialize};
use crate::airflow::client::v1;
use crate::airflow::client::v2;

/// Common Log model used by the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    pub continuation_token: Option<String>,
    pub content: String,
}

// From trait implementations for v1 models
impl From<v1::model::log::Log> for Log {
    fn from(value: v1::model::log::Log) -> Self {
        Log {
            continuation_token: value.continuation_token,
            content: value.content,
        }
    }
}

// From trait implementations for v2 models
impl From<v2::model::log::Log> for Log {
    fn from(value: v2::model::log::Log) -> Self {
        Log {
            continuation_token: value.continuation_token,
            content: value.content.to_string(),
        }
    }
}
