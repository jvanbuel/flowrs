pub mod clear;
pub mod mark;
mod render;
pub mod trigger;

use clear::ClearDagRunPopup;
use mark::MarkDagRunPopup;
use trigger::TriggerDagRunPopUp;

pub enum DagRunPopUp {
    Clear(ClearDagRunPopup),
    Mark(MarkDagRunPopup),
    Trigger(TriggerDagRunPopUp),
}
