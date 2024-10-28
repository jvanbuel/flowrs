use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Log {
    #[serde(rename = "continuation_token")]
    continuation_token: Option<String>,
    content: String,
}
