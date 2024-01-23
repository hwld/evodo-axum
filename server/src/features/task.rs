use std::str::FromStr;
pub mod routers;
pub use routers::router;
use serde::{Deserialize, Serialize};
use strum::EnumString;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema, Debug)]
pub struct Task {
    pub id: String,
    pub status: TaskStatus,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(
    Serialize, Deserialize, ToSchema, EnumString, sqlx::Type, Debug, PartialEq, Clone, Copy,
)]
pub enum TaskStatus {
    Todo,
    Done,
}
impl From<String> for TaskStatus {
    fn from(value: String) -> Self {
        TaskStatus::from_str(value.as_str()).unwrap_or(TaskStatus::Todo)
    }
}

#[derive(Deserialize, ToSchema, Debug)]
pub struct CreateTask {
    #[schema(min_length = 1)]
    pub title: String,
}

#[derive(Deserialize, ToSchema, Debug)]
pub struct UpdateTask {
    pub title: String,
    pub status: TaskStatus,
}
