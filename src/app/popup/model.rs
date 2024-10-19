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
