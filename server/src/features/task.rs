pub mod test;
use std::str::FromStr;
pub mod routers;
use garde::Validate;
pub use routers::router;
use serde::{Deserialize, Serialize};
use strum::EnumString;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
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

#[derive(Serialize, Deserialize, ToSchema, Debug, Validate)]
pub struct CreateTask {
    #[garde(length(min = 1, max = 100))]
    #[schema(min_length = 1, max_length = 100)]
    pub title: String,
}

#[derive(Deserialize, Serialize, ToSchema, Debug, Validate)]
pub struct UpdateTask {
    #[garde(length(min = 1, max = 100))]
    pub title: String,

    #[garde(skip)]
    pub status: TaskStatus,
}
