use super::dagruns::trigger::TriggerDagRunPopUp;

pub mod commands;

pub enum DagPopUp {
    Trigger(TriggerDagRunPopUp),
}
