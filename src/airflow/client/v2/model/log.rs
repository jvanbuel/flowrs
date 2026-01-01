use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

/// An individual structured log message with timestamp and event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredLogMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    pub event: String,
    #[serde(flatten)]
    pub additional_fields: HashMap<String, serde_json::Value>,
}

/// Log content can be either structured messages or plain text lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LogContent {
    Structured(Vec<StructuredLogMessage>),
    Plain(Vec<String>),
}

impl Display for LogContent {
    /// Convert log content to a single string representation
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::Structured(messages) => {
                for msg in messages {
                    if let Some(timestamp) = &msg.timestamp {
                        write!(f, "{timestamp} | ")?;
                    }
                    for (key, value) in &msg.additional_fields {
                        write!(f, "{key}: {value} | ")?;
                    }
                    write!(f, "{} | ", msg.event)?;
                    writeln!(f)?;
                }
            }
            Self::Plain(lines) => {
                for line in lines {
                    writeln!(f, "{line}")?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Log {
    #[serde(rename = "continuation_token")]
    pub continuation_token: Option<String>,
    pub content: LogContent,
}
