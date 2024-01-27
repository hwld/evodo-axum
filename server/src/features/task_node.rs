pub mod routers;
pub mod test;
use garde::Validate;
pub use routers::router;
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
    pub id: String,
    pub task_id: String,
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
