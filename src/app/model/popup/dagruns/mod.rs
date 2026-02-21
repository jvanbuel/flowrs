use clear::ClearDagRunPopup;
use date_filter::DateFilterPopup;
use mark::MarkDagRunPopup;
use trigger::TriggerDagRunPopUp;

pub mod clear;
pub mod commands;
pub mod date_filter;
pub mod mark;
pub mod trigger;

pub enum DagRunPopUp {
    Clear(ClearDagRunPopup),
    Mark(MarkDagRunPopup),
    Trigger(TriggerDagRunPopUp),
    DateFilter(DateFilterPopup),
}
