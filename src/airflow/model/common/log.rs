use serde::{Deserialize, Serialize};

/// Common Log model used by the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    pub continuation_token: Option<String>,
    pub content: String,
}
