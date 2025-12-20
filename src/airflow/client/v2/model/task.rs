use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskCollectionResponse {
    pub tasks: Vec<TaskResponse>,
    pub total_entries: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskResponse {
    pub task_id: String,
    #[serde(default)]
    pub downstream_task_ids: Vec<String>,
    #[serde(default)]
    pub task_display_name: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
}
