use crate::airflow::model::{dag::Dag, dagrun::DagRun, taskinstance::TaskInstance};

enum Action {
    Trigger(Box<Dag>),
    Toggle(Box<Dag>),
    ClearDag(Box<Dag>),
    ClearDagRun(Box<DagRun>),
    ClearTaskInstance(Box<TaskInstance>),
    Refresh,
}
struct ActionBuffer {
    action: Vec<Action>,
}

pub struct PopUp {
    action_buffer: ActionBuffer,
    is_open: bool,
}

impl PopUp {
    pub fn new() -> Self {
        PopUp {
            action_buffer: ActionBuffer { action: vec![] },
            is_open: false,
        }
    }
}
