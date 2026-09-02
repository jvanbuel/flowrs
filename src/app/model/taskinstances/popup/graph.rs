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
#[derive(Debug)]
pub struct GraphNode {
    pub task_id: String,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub border_color: Color,
}

/// Popup that visualizes the DAG task dependency graph.
#[derive(Debug)]
pub struct DagGraphPopup {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<(usize, usize)>,
    pub scroll_x: u16,
    pub scroll_y: u16,
    /// The content height of the graph in rows (used for popup sizing).
    pub content_height: u16,
    /// The content width of the graph in columns (used for scroll clamping).
    pub content_width: u16,
    /// Last known viewport dimensions (set during render, used for scroll clamping).
    viewport: (u16, u16),
}

impl DagGraphPopup {
    /// Build a graph popup from the task graph and current task instance states.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "value is bounded by terminal/layout dimensions and stays well within the target integer range"
    )]
    pub fn new(graph: &TaskGraph, task_instances: &[TaskInstance]) -> Self {
        // Map task_id -> state for coloring
        let state_map: HashMap<&str, &TaskInstanceState> = task_instances
            .iter()
            .filter_map(|ti| ti.state.as_ref().map(|s| (ti.task_id.as_ref(), s)))
            .collect();

        let level_count = if graph.is_empty() {
            0
        } else {
            graph.max_level() + 1
        };

        // Column widths: max node width at each level
        let col_widths: Vec<u16> = (0..level_count)
            .map(|l| {
                graph
                    .tasks_at_level(l)
                    .iter()
                    .map(|t| t.len() as u16 + 2 + 2 * NODE_PADDING)
                    .max()
                    .unwrap_or(0)
            })
            .collect();

        // Column x-positions
        let mut col_x: Vec<u16> = Vec::with_capacity(level_count);
        let mut x = MARGIN;
        for &w in &col_widths {
            col_x.push(x);
            x += w + HORIZONTAL_GAP;
        }

        // Total content dimensions
        let column_height = |task_count: u16| {
            task_count * NODE_HEIGHT + task_count.saturating_sub(1) * VERTICAL_SPACING
        };
        let max_tasks_at_level = (0..level_count)
            .map(|l| graph.level_range(l).len())
            .max()
            .unwrap_or(0) as u16;
        let total_max_height = column_height(max_tasks_at_level);
        let content_height = 2 * MARGIN + total_max_height;
        let content_width = col_x
            .last()
            .zip(col_widths.last())
            .map_or(0, |(&cx, &cw)| cx + cw + MARGIN);

        // Offset to vertically center columns with fewer tasks
        let y_offsets: Vec<u16> = (0..level_count)
            .map(|l| {
                let col_height = column_height(graph.level_range(l).len() as u16);
                total_max_height.saturating_sub(col_height) / 2
            })
            .collect();

        // The graph's dense index order is (level, task_id), which is exactly
        // the order nodes are laid out in, so node `i` is graph task `i` and
        // edges can be copied straight from the graph's index tables.
        let nodes: Vec<GraphNode> = graph
            .ids()
            .iter()
            .enumerate()
            .map(|(i, task_id)| {
                let level = graph.level_at(i);
                let row = (i - graph.level_range(level).start) as u16;
                let y = MARGIN + y_offsets[level] + row * (NODE_HEIGHT + VERTICAL_SPACING);
                let border_color: Color = state_map
                    .get(task_id.as_str())
                    .map_or(Color::DarkGray, |s| AirflowStateColor::from(*s).into());
                GraphNode {
                    task_id: task_id.clone(),
                    x: col_x[level],
                    y,
                    width: col_widths[level],
                    border_color,
                }
            })
            .collect();

        let edges: Vec<(usize, usize)> = (0..graph.len())
            .flat_map(|i| graph.downstream(i).iter().map(move |&to| (i, to)))
            .collect();

        Self {
            nodes,
            edges,
            scroll_x: 0,
            scroll_y: 0,
            content_height,
            content_width,
            viewport: (0, 0),
        }
    }

    /// Store the viewport dimensions from the last render pass so scroll
    /// clamping in [`update`] can prevent scrolling past the content.
    pub fn set_viewport(&mut self, width: u16, height: u16) {
        self.viewport = (width, height);
    }

    /// Maximum horizontal scroll offset that keeps the last column visible.
    fn max_scroll_x(&self) -> u16 {
        self.content_width.saturating_sub(self.viewport.0)
    }

    /// Maximum vertical scroll offset that keeps the last row visible.
    fn max_scroll_y(&self) -> u16 {
        self.content_height.saturating_sub(self.viewport.1)
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
                    self.scroll_x = (self.scroll_x + SCROLL_STEP).min(self.max_scroll_x());
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.scroll_y = self.scroll_y.saturating_sub(SCROLL_STEP);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.scroll_y = (self.scroll_y + SCROLL_STEP).min(self.max_scroll_y());
                }
                _ => {}
            }
        }
        (None, vec![])
    }

    /// Set a cell in the buffer if it falls within the visible area.
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "value is bounded by terminal/layout dimensions and stays well within the target integer range"
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
