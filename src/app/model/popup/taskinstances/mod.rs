use clear::ClearTaskInstancePopup;
use mark::MarkTaskInstancePopup;

pub mod clear;
pub mod commands;
pub mod mark;

pub enum TaskInstancePopUp {
    Clear(ClearTaskInstancePopup),
    Mark(MarkTaskInstancePopup),
}
