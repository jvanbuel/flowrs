use std::collections::HashMap;

use crossterm::event::KeyCode;
use ratatui::style::{Color, Style};

use crate::airflow::graph::TaskGraph;
use crate::airflow::model::common::taskinstance::TaskInstanceState;
use crate::airflow::model::common::TaskInstance;
use crate::app::events::custom::FlowrsEvent;
use crate::app::worker::WorkerMessage;
use crate::ui::constants::AirflowStateColor;

/// Height of each node in rows (top border + content + bottom border).
const NODE_HEIGHT: u16 = 3;
/// Padding inside the node on each side of the task name.
pub const NODE_PADDING: u16 = 1;
/// Vertical gap between nodes at the same level.
const VERTICAL_SPACING: u16 = 2;
/// Horizontal gap between columns for edge routing.
const HORIZONTAL_GAP: u16 = 6;
/// Margin around the entire graph.
const MARGIN: u16 = 1;
/// Scroll step for arrow keys.
const SCROLL_STEP: u16 = 3;

/// A node in the graph layout.
pub struct GraphNode {
    pub task_id: String,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub border_color: Color,
}

/// Popup that visualizes the DAG task dependency graph.
pub struct DagGraphPopup {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<(usize, usize)>,
    pub scroll_x: u16,
    pub scroll_y: u16,
}

impl DagGraphPopup {
    /// Build a graph popup from the task graph and current task instance states.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_possible_wrap
    )]
    pub fn new(graph: &TaskGraph, task_instances: &[TaskInstance]) -> Self {
        // Map task_id -> state for coloring
        let state_map: HashMap<&str, &TaskInstanceState> = task_instances
            .iter()
            .filter_map(|ti| ti.state.as_ref().map(|s| (ti.task_id.as_ref(), s)))
            .collect();

        let max_level = graph.max_level();

        // Gather tasks per level
        let levels: Vec<Vec<String>> = (0..=max_level).map(|l| graph.tasks_at_level(l)).collect();

        // Column widths: max node width at each level
        let col_widths: Vec<u16> = levels
            .iter()
            .map(|tasks| {
                tasks
                    .iter()
                    .map(|t| t.len() as u16 + 2 + 2 * NODE_PADDING)
                    .max()
                    .unwrap_or(0)
            })
            .collect();

        // Column x-positions
        let mut col_x: Vec<u16> = Vec::with_capacity(levels.len());
        let mut x = MARGIN;
        for &w in &col_widths {
            col_x.push(x);
            x += w + HORIZONTAL_GAP;
        }

        // Max tasks at any level (determines canvas height)
        let max_tasks_at_level = levels.iter().map(Vec::len).max().unwrap_or(0) as u16;

        // Build nodes with positions, centering shorter columns vertically
        let mut nodes: Vec<GraphNode> = Vec::new();
        let mut task_to_idx: HashMap<String, usize> = HashMap::new();

        for (level, tasks) in levels.iter().enumerate() {
            let task_count = tasks.len() as u16;
            // Offset to vertically center columns with fewer tasks
            let total_col_height =
                task_count * NODE_HEIGHT + task_count.saturating_sub(1) * VERTICAL_SPACING;
            let total_max_height = max_tasks_at_level * NODE_HEIGHT
                + max_tasks_at_level.saturating_sub(1) * VERTICAL_SPACING;
            let y_offset = total_max_height.saturating_sub(total_col_height) / 2;

            for (row, task_id) in tasks.iter().enumerate() {
                let y = MARGIN + y_offset + row as u16 * (NODE_HEIGHT + VERTICAL_SPACING);
                let width = col_widths[level];
                let border_color: Color = state_map
                    .get(task_id.as_str())
                    .map_or(Color::DarkGray, |s| AirflowStateColor::from(*s).into());

                task_to_idx.insert(task_id.clone(), nodes.len());
                nodes.push(GraphNode {
                    task_id: task_id.clone(),
                    x: col_x[level],
                    y,
                    width,
                    border_color,
                });
            }
        }

        // Build edge list
        let mut edges: Vec<(usize, usize)> = Vec::new();
        for (i, node) in nodes.iter().enumerate() {
            for downstream_id in graph.downstream(&node.task_id) {
                if let Some(&target_idx) = task_to_idx.get(downstream_id.as_str()) {
                    edges.push((i, target_idx));
                }
            }
        }

        Self {
            nodes,
            edges,
            scroll_x: 0,
            scroll_y: 0,
        }
    }

    /// Handle keyboard events (scrolling and dismiss).
    /// Returns a key event on Esc/q to signal the parent to close the popup.
    pub fn update(
        &mut self,
        event: &FlowrsEvent,
        _ctx: &crate::app::state::NavigationContext,
    ) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        if let FlowrsEvent::Key(key) = event {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    return (Some(FlowrsEvent::Key(*key)), vec![]);
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    self.scroll_x = self.scroll_x.saturating_sub(SCROLL_STEP);
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.scroll_x += SCROLL_STEP;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.scroll_y = self.scroll_y.saturating_sub(SCROLL_STEP);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.scroll_y += SCROLL_STEP;
                }
                _ => {}
            }
        }
        (None, vec![])
    }

    /// Set a cell in the buffer if it falls within the visible area.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_possible_wrap
    )]
    pub fn set_cell(
        &self,
        buf: &mut ratatui::buffer::Buffer,
        area: ratatui::layout::Rect,
        canvas_x: i32,
        canvas_y: i32,
        symbol: &str,
        style: Style,
    ) {
        let screen_x = canvas_x - i32::from(self.scroll_x);
        let screen_y = canvas_y - i32::from(self.scroll_y);
        if screen_x >= 0
            && screen_y >= 0
            && screen_x < i32::from(area.width)
            && screen_y < i32::from(area.height)
        {
            let cell = &mut buf[(area.x + screen_x as u16, area.y + screen_y as u16)];
            cell.set_symbol(symbol);
            cell.set_style(style);
        }
    }
}
