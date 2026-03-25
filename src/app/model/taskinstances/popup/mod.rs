pub mod clear;
pub mod graph;
pub mod mark;
mod render;

use clear::ClearTaskInstancePopup;
use graph::DagGraphPopup;
use mark::MarkTaskInstancePopup;

pub enum TaskInstancePopUp {
    Clear(ClearTaskInstancePopup),
    Mark(MarkTaskInstancePopup),
    Graph(DagGraphPopup),
}
