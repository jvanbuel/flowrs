pub(crate) mod params;
mod render;
mod table;
mod text;

use crossterm::event::KeyCode;
use ratatui::widgets::TableState;

use crate::app::{
    events::custom::FlowrsEvent,
    model::{popup::SelectedButton, Model},
    worker::WorkerMessage,
};

use crate::airflow::model::common::DagId;

use params::{build_params, ParamEntry, ParamKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FocusZone {
    Params,
    Buttons,
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
                    // Text, Examples and Enum: open text editor. Cycling
                    // enum options is Space's job, not Enter's.
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
}
