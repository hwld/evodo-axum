pub mod db;
pub mod routes;
pub mod test;
pub mod usecases;
use garde::Validate;
pub use routes::router;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::EnumString;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct Task {
    pub id: String,
    pub status: TaskStatus,
    pub title: String,
    pub description: String,
    pub user_id: String,
    pub sub_task_ids: Vec<String>,
    pub blocked_task_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(
    Serialize, Deserialize, ToSchema, EnumString, sqlx::Type, Debug, PartialEq, Clone, Copy, Default,
)]
pub enum TaskStatus {
    #[default]
    Todo,
    Done,
}
impl From<String> for TaskStatus {
    fn from(value: String) -> Self {
        TaskStatus::from_str(value.as_str()).unwrap_or(TaskStatus::Todo)
    }
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Validate)]
pub struct CreateTask {
    #[garde(length(min = 1, max = 100))]
    #[schema(min_length = 1, max_length = 100)]
    pub title: String,
}

#[derive(Deserialize, Serialize, ToSchema, Debug, Validate)]
pub struct UpdateTask {
    #[garde(length(min = 1, max = 100))]
    #[schema(min_length = 1, max_length = 100)]
    pub title: String,

    #[garde(length(max = 2000))]
    #[schema(max_length = 2000)]
    pub description: String,
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub struct UpdateTaskStatus {
    pub status: TaskStatus,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct DeleteTaskResponse {
    pub task_id: String,
}
