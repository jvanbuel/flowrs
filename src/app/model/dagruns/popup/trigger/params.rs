//! Parsing of a raw Airflow DAG `params` schema into editable entries.

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ParamKind {
    /// Free-text input
    Text,
    /// Boolean toggle (true/false)
    Bool,
    /// Fixed set of allowed values (from schema.enum)
    Enum(Vec<String>),
    /// Suggested values but free-text also allowed (from schema.examples)
    Examples(Vec<String>),
}

pub(crate) struct ParamEntry {
    pub key: String,
    pub value: String,
    pub description: Option<String>,
    pub kind: ParamKind,
    pub json_valid: bool,
}

impl ParamEntry {
    pub fn options(&self) -> &[String] {
        match &self.kind {
            ParamKind::Enum(opts) | ParamKind::Examples(opts) => opts,
            _ => &[],
        }
    }

    /// Recompute `json_valid` for the current value.
    ///
    /// Only free-text params (plain text or examples-with-suggestions) that
    /// look like a structured JSON literal (object or array) can be "invalid"
    /// — bools/enums are machine-controlled and a plain string value is
    /// legitimately sent as a JSON string, so neither is ever flagged.
    pub(crate) fn revalidate(&mut self) {
        self.json_valid = match self.kind {
            ParamKind::Text | ParamKind::Examples(_) if looks_like_json_struct(&self.value) => {
                serde_json::from_str::<serde_json::Value>(&self.value).is_ok()
            }
            _ => true,
        };
    }
}

/// Whether `value` looks like it was meant to be a structured JSON literal.
fn looks_like_json_struct(value: &str) -> bool {
    let trimmed = value.trim_start();
    trimmed.starts_with('{') || trimmed.starts_with('[')
}

/// Build the editable param list from a raw DAG `params` schema object.
pub(crate) fn build_params(raw_params: Option<&serde_json::Value>) -> Vec<ParamEntry> {
    raw_params
        .and_then(|v| v.as_object())
        .map(|obj| obj.iter().map(|(k, v)| extract_param(k, v)).collect())
        .unwrap_or_default()
}

fn extract_param(key: &str, v: &serde_json::Value) -> ParamEntry {
    let Some(obj) = v.as_object() else {
        return ParamEntry {
            key: key.to_owned(),
            value: value_to_string(v),
            description: None,
            kind: kind_from_value(v),
            json_valid: true,
        };
    };

    // Airflow serializes each Param as an object carrying its `value`, `schema`
    // and `description`. The legacy `__class` tag is present in some versions
    // and absent in others, so detect the wrapper by any of its structural
    // fields rather than by `__class` alone.
    let is_wrapped_param =
        obj.contains_key("__class") || obj.contains_key("schema") || obj.contains_key("value");
    if is_wrapped_param {
        let raw_value = obj.get("value").filter(|v| !v.is_null());
        let value = raw_value.map_or_else(String::new, value_to_string);

        let description = obj
            .get("description")
            .and_then(serde_json::Value::as_str)
            .map(String::from);

        let schema = obj.get("schema");
        let kind = kind_from_schema(schema, raw_value);

        return ParamEntry {
            key: key.to_owned(),
            value,
            description,
            kind,
            json_valid: true,
        };
    }

    // Some Airflow versions use a "default" key
    if let Some(default) = obj.get("default") {
        return ParamEntry {
            key: key.to_owned(),
            value: value_to_string(default),
            description: None,
            kind: kind_from_value(default),
            json_valid: true,
        };
    }

    ParamEntry {
        key: key.to_owned(),
        value: value_to_string(v),
        description: None,
        kind: ParamKind::Text,
        json_valid: true,
    }
}

fn kind_from_value(v: &serde_json::Value) -> ParamKind {
    if v.is_boolean() {
        ParamKind::Bool
    } else {
        ParamKind::Text
    }
}

fn kind_from_schema(
    schema: Option<&serde_json::Value>,
    raw_value: Option<&serde_json::Value>,
) -> ParamKind {
    let Some(schema) = schema.and_then(|s| s.as_object()) else {
        return raw_value.map_or(ParamKind::Text, kind_from_value);
    };

    // Check for enum (closed set)
    if let Some(values) = schema.get("enum").and_then(|v| v.as_array()) {
        let opts: Vec<String> = values.iter().map(value_to_string).collect();
        if !opts.is_empty() {
            return ParamKind::Enum(opts);
        }
    }

    // Check for examples (open set with suggestions)
    if let Some(values) = schema.get("examples").and_then(|v| v.as_array()) {
        let opts: Vec<String> = values.iter().map(value_to_string).collect();
        if !opts.is_empty() {
            return ParamKind::Examples(opts);
        }
    }

    // Check schema type for booleans
    if schema
        .get("type")
        .and_then(|t| t.as_str())
        .is_some_and(|t| t == "boolean")
    {
        return ParamKind::Bool;
    }

    raw_value.map_or(ParamKind::Text, kind_from_value)
}

fn value_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(_) | serde_json::Value::Number(_) => v.to_string(),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            serde_json::to_string(v).unwrap_or_default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_wrapped_param_without_class_tag() {
        // The shape returned by Airflow instances that omit `__class`.
        let entry = extract_param(
            "awi",
            &serde_json::json!({
                "value": false,
                "description": "Resync table: awi",
                "schema": { "type": "boolean" }
            }),
        );
        assert_eq!(entry.value, "false");
        assert_eq!(entry.kind, ParamKind::Bool);
        assert_eq!(entry.description.as_deref(), Some("Resync table: awi"));
    }

    #[test]
    fn wrapped_param_value_is_not_dumped_as_json() {
        // Regression: the whole wrapper object must not become the value text.
        let entry = extract_param(
            "schooljaar_start",
            &serde_json::json!({
                "value": 2020,
                "description": "Start of the schoolyear range",
                "schema": { "type": "integer" }
            }),
        );
        assert_eq!(entry.value, "2020");
        assert!(!entry.value.contains("schema"));
    }
}
