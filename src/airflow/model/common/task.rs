use serde::{Deserialize, Serialize};

use crate::airflow::client::{v1, v2};

/// Common Task model representing a task definition in a DAG
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub task_id: String,
    pub downstream_task_ids: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskList {
    pub tasks: Vec<Task>,
}

impl From<v1::model::task::TaskResponse> for Task {
    fn from(value: v1::model::task::TaskResponse) -> Self {
        Task {
            task_id: value.task_id,
            downstream_task_ids: value.downstream_task_ids,
        }
    }
}

impl From<v1::model::task::TaskCollectionResponse> for TaskList {
    fn from(value: v1::model::task::TaskCollectionResponse) -> Self {
        TaskList {
            tasks: value.tasks.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<v2::model::task::TaskResponse> for Task {
    fn from(value: v2::model::task::TaskResponse) -> Self {
        Task {
            task_id: value.task_id,
            downstream_task_ids: value.downstream_task_ids,
        }
    }
}

impl From<v2::model::task::TaskCollectionResponse> for TaskList {
    fn from(value: v2::model::task::TaskCollectionResponse) -> Self {
        TaskList {
            tasks: value.tasks.into_iter().map(Into::into).collect(),
        }
    }
}
