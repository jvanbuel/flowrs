pub mod commands;
mod dag_code_view;
pub mod popup;
mod render;

use crossterm::event::KeyCode;
use log::debug;
use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::airflow::model::common::{DagRun, DagRunId};
use crate::app::events::custom::FlowrsEvent;
use crate::app::navigation::Selection;

use super::{FilterableTable, KeyResult, Model, Popup};
use crate::airflow::model::common::OpenItem;
use crate::app::worker::WorkerMessage;
use commands::DAGRUN_COMMAND_POP_UP;
use popup::clear::ClearDagRunPopup;
use popup::mark::MarkDagRunPopup;
use popup::DagRunPopUp;

pub use dag_code_view::DagCodeView;

/// Model for the DAG Run panel, managing the list of DAG runs and their filtering.
#[derive(Debug)]
pub struct DagRunModel {
    pub dag_code: Option<DagCodeView>,
    /// Filterable table containing all DAG runs and filtered view
    pub table: FilterableTable<DagRun>,
    /// Unified popup state (error, commands, or custom for this model)
    pub popup: Popup<DagRunPopUp>,
    ticks: u32,
    poll_tick_multiplier: u32,
    event_buffer: Vec<KeyCode>,
}

impl Default for DagRunModel {
    fn default() -> Self {
        Self {
            dag_code: None,
            table: FilterableTable::new(),
            popup: Popup::None,
            ticks: 0,
            poll_tick_multiplier: 10,
            event_buffer: Vec::new(),
        }
    }
}

impl DagRunModel {
    pub fn new(poll_tick_multiplier: u32) -> Self {
        Self {
            poll_tick_multiplier,
            ..Self::default()
        }
    }

    /// Create a text-based duration gauge line.
    /// The gauge normalizes durations to show relative progress within visible items.
    pub(crate) fn create_duration_gauge(
        duration_seconds: f64,
        max_duration: f64,
        color: ratatui::style::Color,
        width: usize,
    ) -> Line<'static> {
        const FILLED_CHAR: &str = "▃";
        const EMPTY_CHAR: &str = " ";

        // Calculate the ratio (0.0 to 1.0)
        let ratio = if max_duration > 0.0 {
            (duration_seconds / max_duration).min(1.0)
        } else {
            0.0
        };

        // Calculate how many characters should be filled
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss,
            reason = "value is bounded by terminal/layout dimensions and stays well within the target integer range"
        )]
        let filled_width = (ratio * width as f64).round() as usize;
        let empty_width = width.saturating_sub(filled_width);

        // Create the gauge string
        let filled = FILLED_CHAR.repeat(filled_width);
        let empty = EMPTY_CHAR.repeat(empty_width);

        Line::from(vec![
            Span::styled(filled, Style::default().fg(color).bold()),
            Span::styled(empty, Style::default().fg(color).dim()),
        ])
    }

    /// Get the currently selected DAG run
    pub fn current(&self) -> Option<&DagRun> {
        self.table.current()
    }

    /// The DAG run under the cursor, as a navigation selection.
    pub fn selection(&self) -> Option<Selection> {
        self.current()
            .map(|dag_run| Selection::DagRun(dag_run.dag_run_id.clone()))
    }

    /// Forget view state tied to the previously selected DAG.
    pub fn reset_view(&mut self) {
        self.table.reset_cursor();
    }

    /// Returns selected DAG run IDs for passing to mark/clear popups
    fn selected_dag_run_ids(&self) -> Vec<DagRunId> {
        self.table.selected_ids(|item| item.dag_run_id.clone())
    }
}

impl DagRunModel {
    /// Handle dag code viewer navigation
    fn handle_dag_code_viewer(&mut self, key_code: KeyCode) -> KeyResult {
        let Some(view) = self.dag_code.as_mut() else {
            return KeyResult::Ignored;
        };
        if view.update(key_code) {
            self.dag_code = None;
        }
        KeyResult::Consumed
    }

    /// Handle model-specific popups (returns messages from popup)
    fn handle_popup(
        &mut self,
        event: &FlowrsEvent,
        ctx: &crate::app::navigation::Path,
    ) -> Option<Vec<WorkerMessage>> {
        let custom_popup = self.popup.custom_mut()?;
        let (key_event, messages) = match custom_popup {
            DagRunPopUp::Clear(p) => p.update(event, ctx),
            DagRunPopUp::Mark(p) => p.update(event, ctx),
            DagRunPopUp::Trigger(p) => p.update(event, ctx),
            DagRunPopUp::Graph(p) => p.update(event, ctx),
        };
        debug!("Popup messages: {messages:?}");

        if let Some(FlowrsEvent::Key(key_event)) = &key_event {
            if matches!(
                key_event.code,
                KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q')
            ) {
                let exit_visual =
                    matches!(custom_popup, DagRunPopUp::Clear(_) | DagRunPopUp::Mark(_));
                self.popup.close();
                if exit_visual {
                    self.table.clear_visual_mode();
                }
            }
        }
        Some(messages)
    }

    /// Handle model-specific keys
    fn handle_keys(&mut self, key_code: KeyCode, ctx: &crate::app::navigation::Path) -> KeyResult {
        match key_code {
            KeyCode::Char('t') => {
                if let Some(dag_id) = ctx.dag_id() {
                    // The worker fetches a fresh param schema (falling back to
                    // the cached one) and opens the trigger popup once ready.
                    return KeyResult::ConsumedWith(vec![WorkerMessage::GetDagParams {
                        dag_id: dag_id.clone(),
                    }]);
                }
                KeyResult::Consumed
            }
            KeyCode::Char('m') => {
                let dag_run_ids = self.selected_dag_run_ids();
                if let Some(dag_id) = ctx.dag_id() {
                    if !dag_run_ids.is_empty() {
                        self.popup
                            .show_custom(DagRunPopUp::Mark(MarkDagRunPopup::new(
                                dag_run_ids,
                                dag_id.clone(),
                            )));
                    }
                }
                KeyResult::Consumed
            }
            KeyCode::Char('?') => {
                self.popup.show_commands(&DAGRUN_COMMAND_POP_UP);
                KeyResult::Consumed
            }
            KeyCode::Char('d') => {
                if let (Some(dag_id), Some(dag_run)) = (ctx.dag_id(), &self.current()) {
                    KeyResult::ConsumedWith(vec![WorkerMessage::ShowDagGraph {
                        dag_id: dag_id.clone(),
                        dag_run_id: dag_run.dag_run_id.clone(),
                    }])
                } else {
                    KeyResult::Consumed
                }
            }
            KeyCode::Char('v') => {
                if let Some(dag_id) = ctx.dag_id() {
                    KeyResult::ConsumedWith(vec![WorkerMessage::GetDagCode {
                        dag_id: dag_id.clone(),
                    }])
                } else {
                    KeyResult::Consumed
                }
            }
            KeyCode::Char('c') => {
                let dag_run_ids = self.selected_dag_run_ids();
                if let Some(dag_id) = ctx.dag_id() {
                    if !dag_run_ids.is_empty() {
                        self.popup
                            .show_custom(DagRunPopUp::Clear(ClearDagRunPopup::new(
                                dag_run_ids,
                                dag_id.clone(),
                            )));
                    }
                }
                KeyResult::Consumed
            }
            KeyCode::Char('o') => {
                if let (Some(dag_id), Some(dag_run)) = (ctx.dag_id(), &self.current()) {
                    KeyResult::PassWith(vec![WorkerMessage::OpenItem(OpenItem::DagRun {
                        dag_id: dag_id.clone(),
                        dag_run_id: dag_run.dag_run_id.clone(),
                    })])
                } else {
                    KeyResult::Consumed
                }
            }
            _ => KeyResult::PassThrough,
        }
    }
}

impl Model for DagRunModel {
    fn update(
        &mut self,
        event: &FlowrsEvent,
        ctx: &crate::app::navigation::Path,
    ) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        match event {
            FlowrsEvent::Tick => {
                self.ticks += 1;
                if !self.ticks.is_multiple_of(self.poll_tick_multiplier) {
                    return (Some(FlowrsEvent::Tick), vec![]);
                }
                let worker_messages = if let Some(dag_id) = ctx.dag_id() {
                    vec![WorkerMessage::UpdateDagRuns {
                        dag_id: dag_id.clone(),
                    }]
                } else {
                    Vec::default()
                };
                (Some(FlowrsEvent::Tick), worker_messages)
            }
            FlowrsEvent::Key(key_event) => {
                if matches!(
                    self.table.handle_filter_key(key_event),
                    KeyResult::Consumed | KeyResult::ConsumedWith(_)
                ) {
                    return (None, vec![]);
                }

                // Popup handling (has its own update method)
                if let Some(messages) = self.handle_popup(event, ctx) {
                    return (None, messages);
                }

                // Chain the remaining handlers
                let result = self
                    .popup
                    .handle_dismiss(key_event.code)
                    .or_else(|| self.handle_dag_code_viewer(key_event.code))
                    .or_else(|| self.table.handle_visual_mode_key(key_event.code))
                    .or_else(|| {
                        self.table
                            .handle_navigation(key_event.code, &mut self.event_buffer)
                    })
                    .or_else(|| self.handle_keys(key_event.code, ctx));

                result.into_result(event)
            }
            FlowrsEvent::Mouse | FlowrsEvent::FocusGained | FlowrsEvent::FocusLost => {
                (Some(event.clone()), vec![])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_gauge_ratios() {
        let gauge_full =
            DagRunModel::create_duration_gauge(100.0, 100.0, ratatui::style::Color::Green, 10);
        let gauge_half =
            DagRunModel::create_duration_gauge(50.0, 100.0, ratatui::style::Color::Green, 10);
        let gauge_empty =
            DagRunModel::create_duration_gauge(0.0, 100.0, ratatui::style::Color::Green, 10);

        assert_eq!(gauge_full.spans[0].content.chars().count(), 10);
        assert_eq!(gauge_half.spans[0].content.chars().count(), 5);
        assert_eq!(gauge_empty.spans[0].content.chars().count(), 0);
    }

    #[test]
    fn test_duration_gauge_edge_cases() {
        // Zero max should not panic
        let gauge = DagRunModel::create_duration_gauge(50.0, 0.0, ratatui::style::Color::Green, 10);
        assert_eq!(gauge.spans[0].content.chars().count(), 0);

        // Duration exceeding max should cap at 100%
        let gauge =
            DagRunModel::create_duration_gauge(150.0, 100.0, ratatui::style::Color::Green, 10);
        assert_eq!(gauge.spans[0].content.chars().count(), 10);
    }
}
