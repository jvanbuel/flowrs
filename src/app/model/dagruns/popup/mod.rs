pub mod clear;
pub mod mark;
pub mod trigger;

use clear::ClearDagRunPopup;
use mark::MarkDagRunPopup;
use trigger::TriggerDagRunPopUp;

use crate::app::model::taskinstances::popup::graph::DagGraphPopup;

pub enum DagRunPopUp {
    Clear(ClearDagRunPopup),
    Mark(MarkDagRunPopup),
    Trigger(TriggerDagRunPopUp),
    Graph(DagGraphPopup),
}
