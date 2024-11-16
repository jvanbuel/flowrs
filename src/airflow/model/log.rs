use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Log {
    #[serde(rename = "continuation_token")]
    pub continuation_token: Option<String>,
    pub content: String,
}
