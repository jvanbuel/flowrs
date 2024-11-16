use clear::ClearDagRunPopup;
use mark::MarkDagRunPopup;
use trigger::TriggerDagRunPopUp;

pub mod clear;
pub mod commands;
pub mod mark;
pub mod trigger;

pub enum DagRunPopUp {
    Clear(ClearDagRunPopup),
    Mark(MarkDagRunPopup),
    Trigger(TriggerDagRunPopUp),
}
