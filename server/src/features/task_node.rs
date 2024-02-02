pub mod db;
pub mod routes;
pub mod test;
use garde::Validate;
pub use routes::router;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::task::{CreateTask, Task};

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct TaskNode {
    pub task: Task,
    pub node_info: TaskNodeInfo,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct TaskNodeInfo {
    pub task_id: String,
    pub user_id: String,
    // サブノードのid
    // TODO: これはTask::subtask_idsで代用できる
    pub subnode_ids: Vec<String>,
    /// すべての祖先のNodeId
    pub ancestor_ids: Vec<String>,
    pub x: f64,
    pub y: f64,
}

#[derive(Deserialize, Serialize, ToSchema, Debug, Validate)]
pub struct CreateTaskNode {
    #[garde(skip)]
    x: f64,
    #[garde(skip)]
    y: f64,
    #[garde(dive)]
    task: CreateTask,
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub struct UpdateTaskNodeInfo {
    pub x: f64,
    pub y: f64,
}
