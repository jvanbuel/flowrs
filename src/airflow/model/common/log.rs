use crate::airflow::client::v1;
use crate::airflow::client::v2;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

/// Compiled regex for parsing V1 log content (Python tuple format).
///
/// Matches tuples like `('hostname', 'log content')` or `('hostname', "log content")`
/// where the second element can use either single or double quotes.
static V1_LOG_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\(\s*'((?:\\.|[^'])*)'\s*,\s*(?:"((?:\\.|[^"])*)"|'((?:\\.|[^'])*)')\s*\)"#)
        .expect("V1 log parsing regex pattern should be valid")
});

/// Common Log model used by the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    pub continuation_token: Option<String>,
    pub content: String,
}

/// Parses V1 log content from Python tuple format to plain text.
///
/// V1 Airflow logs come as serialized Python tuples: `[('host', 'log line\nmore')]`
/// This extracts the log text, joins multiple tuples, and expands escaped newlines.
fn parse_v1_log_content(content: &str) -> String {
    let fragments: Vec<String> = V1_LOG_REGEX
        .captures_iter(content)
        .map(|cap| {
            // Second element can be in group 2 (double quotes) or group 3 (single quotes)
            cap.get(2)
                .or_else(|| cap.get(3))
                .map(|m| m.as_str().to_string())
                .unwrap_or_default()
        })
        .collect();

    if fragments.is_empty() {
        // Not V1 format, return as-is
        content.to_string()
    } else {
        // Join fragments and expand escaped newlines
        fragments
            .into_iter()
            .flat_map(|fragment| {
                fragment
                    .replace("\\n", "\n")
                    .lines()
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// From trait implementations for v1 models
impl From<v1::model::log::Log> for Log {
    fn from(value: v1::model::log::Log) -> Self {
        Self {
            continuation_token: value.continuation_token,
            content: parse_v1_log_content(&value.content),
        }
    }
}

// From trait implementations for v2 models
impl From<v2::model::log::Log> for Log {
    fn from(value: v2::model::log::Log) -> Self {
        Self {
            continuation_token: value.continuation_token,
            content: value.content.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_v1_single_quotes() {
        let content = "[('host1', 'log content here')]";
        let result = parse_v1_log_content(content);
        assert_eq!(result, "log content here");
    }

    #[test]
    fn test_parse_v1_double_quotes() {
        let content = r#"[('cec849a302e3', "*** Found local files")]"#;
        let result = parse_v1_log_content(content);
        assert_eq!(result, "*** Found local files");
    }

    #[test]
    fn test_parse_v1_escaped_newlines_expanded() {
        let content = r#"[('host', "line1\nline2\nline3")]"#;
        let result = parse_v1_log_content(content);
        assert_eq!(result, "line1\nline2\nline3");
    }

    #[test]
    fn test_parse_v1_multiple_tuples_joined() {
        let content = r#"[('host1', 'log1'), ('host2', "log2")]"#;
        let result = parse_v1_log_content(content);
        assert_eq!(result, "log1\nlog2");
    }

    #[test]
    fn test_parse_v1_real_airflow_log() {
        let content = r#"[('cec849a302e3', "*** Found local files:\n***   * /opt/airflow/logs/dag.log\n[2025-10-12T01:24:16.754+0000] INFO - Pre task")]"#;
        let result = parse_v1_log_content(content);
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "*** Found local files:");
        assert_eq!(lines[1], "***   * /opt/airflow/logs/dag.log");
        assert!(lines[2].contains("INFO - Pre task"));
    }

    #[test]
    fn test_parse_v1_plain_text_passthrough() {
        // Non-V1 format should pass through unchanged
        let content = "Just some plain log text\nwith multiple lines";
        let result = parse_v1_log_content(content);
        assert_eq!(result, content);
    }

    #[test]
    fn test_parse_v1_with_escaped_quotes_in_content() {
        let content = r"[('host', 'line with \' escaped quote')]";
        let result = parse_v1_log_content(content);
        assert_eq!(result, r"line with \' escaped quote");
    }
}
