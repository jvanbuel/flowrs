use super::{DagId, DagRunId, TaskId};

#[derive(Debug)]
pub enum OpenItem {
    Config(String),
    Dag {
        dag_id: DagId,
    },
    DagRun {
        dag_id: DagId,
        dag_run_id: DagRunId,
    },
    TaskInstance {
        dag_id: DagId,
        dag_run_id: DagRunId,
        task_id: TaskId,
    },
    Log {
        dag_id: DagId,
        dag_run_id: DagRunId,
        task_id: TaskId,
        #[allow(dead_code)]
        task_try: u32,
    },
}
