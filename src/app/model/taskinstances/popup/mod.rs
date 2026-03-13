pub mod clear;
pub mod mark;
mod render;

use clear::ClearTaskInstancePopup;
use mark::MarkTaskInstancePopup;

pub enum TaskInstancePopUp {
    Clear(ClearTaskInstancePopup),
    Mark(MarkTaskInstancePopup),
}
