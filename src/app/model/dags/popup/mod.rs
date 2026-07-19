use crate::app::model::dagruns::popup::trigger::TriggerDagRunPopUp;

#[derive(Debug)]
pub enum DagPopUp {
    Trigger(TriggerDagRunPopUp),
}
