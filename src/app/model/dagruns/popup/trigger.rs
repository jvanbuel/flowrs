use crossterm::event::KeyCode;
use ratatui::widgets::TableState;

use crate::app::{
    events::custom::FlowrsEvent,
    model::{popup::SelectedButton, Model},
    worker::WorkerMessage,
};

use crate::airflow::model::common::DagId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FocusZone {
    Params,
    Buttons,
}

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
    /// Only free-text params that look like a structured JSON literal (object
    /// or array) can be "invalid" — bools/enums are machine-controlled and a
    /// plain string value is legitimately sent as a JSON string, so neither is
    /// ever flagged.
    fn revalidate(&mut self) {
        self.json_valid = match self.kind {
            ParamKind::Text if looks_like_json_struct(&self.value) => {
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

pub struct TriggerDagRunPopUp {
    pub dag_id: DagId,
    pub(crate) params: Vec<ParamEntry>,
    pub(crate) active_param: usize,
    pub(crate) editing: bool,
    pub(crate) cursor_pos: usize,
    /// Table selection/scroll state; the row scroll offset is managed by
    /// ratatui's `Table` widget, driven from `active_param` at render time.
    pub(crate) table_state: TableState,
    pub(crate) focus: FocusZone,
    pub(crate) selected_button: SelectedButton,
}

impl TriggerDagRunPopUp {
    pub fn new(dag_id: DagId, raw_params: Option<&serde_json::Value>) -> Self {
        Self {
            dag_id,
            params: build_params(raw_params),
            active_param: 0,
            editing: false,
            cursor_pos: 0,
            table_state: TableState::default(),
            focus: FocusZone::Params,
            selected_button: SelectedButton::default(),
        }
    }

    pub(crate) fn has_params(&self) -> bool {
        !self.params.is_empty()
    }

    fn build_conf_and_validate(&mut self) -> Option<serde_json::Value> {
        if self.params.is_empty() {
            return None;
        }
        let mut map = serde_json::Map::new();
        for entry in &mut self.params {
            entry.revalidate();
            let parsed = serde_json::from_str(&entry.value)
                .unwrap_or_else(|_| serde_json::Value::String(entry.value.clone()));
            map.insert(entry.key.clone(), parsed);
        }
        Some(serde_json::Value::Object(map))
    }

    pub(crate) fn active_entry(&self) -> Option<&ParamEntry> {
        self.params.get(self.active_param)
    }

    fn cycle_option(&mut self, forward: bool) {
        let Some(entry) = self.params.get_mut(self.active_param) else {
            return;
        };
        let (ParamKind::Enum(opts) | ParamKind::Examples(opts)) = &entry.kind else {
            return;
        };
        if opts.is_empty() {
            return;
        }
        let current_idx = opts.iter().position(|o| *o == entry.value).unwrap_or(0);
        let next_idx = if forward {
            (current_idx + 1) % opts.len()
        } else {
            current_idx.checked_sub(1).unwrap_or(opts.len() - 1)
        };
        entry.value = opts[next_idx].clone();
    }

    fn toggle_bool(&mut self) {
        let Some(entry) = self.params.get_mut(self.active_param) else {
            return;
        };
        if entry.kind == ParamKind::Bool {
            entry.value = if entry.value == "true" {
                "false"
            } else {
                "true"
            }
            .to_string();
        }
    }
}

/// Build the editable param list from a raw DAG `params` schema object.
fn build_params(raw_params: Option<&serde_json::Value>) -> Vec<ParamEntry> {
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

pub(crate) fn value_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(_) | serde_json::Value::Number(_) => v.to_string(),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            serde_json::to_string(v).unwrap_or_default()
        }
    }
}

impl Model for TriggerDagRunPopUp {
    fn update(
        &mut self,
        event: &FlowrsEvent,
        _ctx: &crate::app::state::NavigationContext,
    ) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        if let FlowrsEvent::Key(key_event) = event {
            if self.has_params() {
                return self.update_with_params(key_event.code, *key_event);
            }
            return self.update_simple(key_event.code, *key_event);
        }
        (Some(event.clone()), vec![])
    }
}

impl TriggerDagRunPopUp {
    fn update_simple(
        &mut self,
        code: KeyCode,
        key_event: crossterm::event::KeyEvent,
    ) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        match code {
            KeyCode::Enter => {
                // On Enter, we always return the key event, so the parent can close the popup
                // If Yes is selected, we also return a WorkerMessage to trigger the dag run
                if self.selected_button.is_yes() {
                    return (
                        Some(FlowrsEvent::Key(key_event)),
                        vec![WorkerMessage::TriggerDagRun {
                            dag_id: self.dag_id.clone(),
                            conf: None,
                        }],
                    );
                }
                return (Some(FlowrsEvent::Key(key_event)), vec![]);
            }
            KeyCode::Char('j' | 'k' | 'h' | 'l')
            | KeyCode::Down
            | KeyCode::Up
            | KeyCode::Left
            | KeyCode::Right => {
                self.selected_button.toggle();
                return (None, vec![]);
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                // On Esc, we always return the key event, so the parent can close the popup, without clearing the dag run
                return (Some(FlowrsEvent::Key(key_event)), vec![]);
            }
            _ => {}
        }
        (Some(FlowrsEvent::Key(key_event)), vec![])
    }

    fn update_with_params(
        &mut self,
        code: KeyCode,
        key_event: crossterm::event::KeyEvent,
    ) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        if self.editing {
            return self.handle_editing(code);
        }

        match code {
            // Esc on the buttons returns to param editing; Esc on the params
            // (or `q` anywhere) closes the popup.
            KeyCode::Esc if self.focus == FocusZone::Buttons => {
                self.focus = FocusZone::Params;
                (None, vec![])
            }
            KeyCode::Esc | KeyCode::Char('q') => (Some(FlowrsEvent::Key(key_event)), vec![]),
            KeyCode::Tab | KeyCode::BackTab => {
                self.focus = match self.focus {
                    FocusZone::Params => FocusZone::Buttons,
                    FocusZone::Buttons => FocusZone::Params,
                };
                (None, vec![])
            }
            KeyCode::Enter => {
                if self.focus == FocusZone::Buttons {
                    if self.selected_button.is_yes() {
                        return (
                            Some(FlowrsEvent::Key(key_event)),
                            vec![WorkerMessage::TriggerDagRun {
                                dag_id: self.dag_id.clone(),
                                conf: self.build_conf_and_validate(),
                            }],
                        );
                    }
                    return (Some(FlowrsEvent::Key(key_event)), vec![]);
                }
                let value_len = self.active_entry().map_or(0, |e| e.value.len());
                match self.active_entry().map(|e| &e.kind) {
                    // Bool: toggle on Enter instead of opening text editor
                    Some(ParamKind::Bool) => self.toggle_bool(),
                    // Enum: cycle on Enter (no free-text editing)
                    Some(ParamKind::Enum(_)) => self.cycle_option(true),
                    // Text and Examples: open text editor
                    Some(_) => {
                        self.editing = true;
                        self.cursor_pos = value_len;
                    }
                    None => {}
                }
                (None, vec![])
            }
            KeyCode::Char(' ') if self.focus == FocusZone::Params => {
                // Space toggles bools and cycles enums/examples for quick editing
                match self.active_entry().map(|e| &e.kind) {
                    Some(ParamKind::Bool) => self.toggle_bool(),
                    Some(ParamKind::Enum(_) | ParamKind::Examples(_)) => self.cycle_option(true),
                    _ => {}
                }
                (None, vec![])
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.focus == FocusZone::Params && !self.params.is_empty() {
                    self.active_param = (self.active_param + 1).min(self.params.len() - 1);
                }
                (None, vec![])
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.focus == FocusZone::Params {
                    self.active_param = self.active_param.saturating_sub(1);
                }
                (None, vec![])
            }
            KeyCode::Char('h' | 'l') | KeyCode::Left | KeyCode::Right => {
                if self.focus == FocusZone::Buttons {
                    self.selected_button.toggle();
                }
                (None, vec![])
            }
            _ => (None, vec![]),
        }
    }

    fn handle_editing(&mut self, code: KeyCode) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        let Some(entry) = self.params.get_mut(self.active_param) else {
            return (None, vec![]);
        };
        let value = &mut entry.value;
        match code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Tab | KeyCode::BackTab => {
                self.editing = false;
            }
            KeyCode::Char(c) => {
                debug_assert!(
                    value.is_char_boundary(self.cursor_pos),
                    "cursor not on char boundary"
                );
                value.insert(self.cursor_pos, c);
                self.cursor_pos += c.len_utf8();
            }
            KeyCode::Backspace if self.cursor_pos > 0 => {
                let prev = value[..self.cursor_pos]
                    .char_indices()
                    .next_back()
                    .map_or(0, |(i, _)| i);
                value.replace_range(prev..self.cursor_pos, "");
                self.cursor_pos = prev;
            }
            KeyCode::Left if self.cursor_pos > 0 => {
                self.cursor_pos = value[..self.cursor_pos]
                    .char_indices()
                    .next_back()
                    .map_or(0, |(i, _)| i);
            }
            KeyCode::Right if self.cursor_pos < value.len() => {
                self.cursor_pos = value[self.cursor_pos..]
                    .char_indices()
                    .nth(1)
                    .map_or(value.len(), |(i, _)| self.cursor_pos + i);
            }
            _ => {}
        }
        // Keep the JSON-validity hint in sync as the user types.
        entry.revalidate();
        (None, vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn esc(focus: FocusZone) -> (Option<FlowrsEvent>, FocusZone) {
        let schema = serde_json::json!({ "x": { "value": 1, "schema": { "type": "integer" } } });
        let mut popup = TriggerDagRunPopUp::new(DagId::from("d"), Some(&schema));
        popup.focus = focus;
        let key = crossterm::event::KeyEvent::from(KeyCode::Esc);
        let (event, _) = popup.update_with_params(KeyCode::Esc, key);
        (event, popup.focus)
    }

    #[test]
    fn esc_on_buttons_returns_to_params_without_closing() {
        let (event, focus) = esc(FocusZone::Buttons);
        assert!(event.is_none(), "popup must stay open");
        assert_eq!(focus, FocusZone::Params);
    }

    #[test]
    fn esc_on_params_closes_the_popup() {
        let (event, _) = esc(FocusZone::Params);
        assert!(
            event.is_some(),
            "Esc on params returns the key so the parent closes it"
        );
    }

    #[test]
    fn tab_exits_edit_mode() {
        let schema = serde_json::json!({ "x": { "value": "hi", "schema": { "type": "string" } } });
        let mut popup = TriggerDagRunPopUp::new(DagId::from("d"), Some(&schema));
        popup.editing = true;
        popup.handle_editing(KeyCode::Tab);
        assert!(!popup.editing);
    }

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
