use clear::ClearDagRunPopup;
use mark::MarkDagRunPopup;

pub mod clear;
pub mod mark;

pub enum DagRunPopUp {
    Clear(ClearDagRunPopup),
    Mark(MarkDagRunPopup),
    Trigger,
}
