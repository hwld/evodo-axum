#[cfg(test)]
mod factory;
pub mod routers;
use garde::Unvalidated;
pub use routers::router;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::task::{CreateTask, Task};

#[derive(Serialize, ToSchema, Debug)]
pub struct TaskNode {
    pub task: Task,
    pub node_info: TaskNodeInfo,
}

#[derive(Serialize, ToSchema, Debug)]
pub struct TaskNodeInfo {
    pub id: String,
    pub task_id: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Deserialize, ToSchema, Debug)]
pub struct CreateTaskNode {
    x: f64,
    y: f64,
    #[schema(value_type = CreateTask)]
    task: Unvalidated<CreateTask>,
}

#[derive(Deserialize, ToSchema, Debug)]
pub struct UpdateTaskNodeInfo {
    pub x: f64,
    pub y: f64,
}
