use crate::AppState;
use axum::{
    routing::{get, put},
    Router,
};
pub mod create_task;
pub mod delete_task;
pub mod get_tasks;
pub mod update_task;

pub const TAG: &str = "task";

pub const TASKS_PATH: &str = "/tasks";
pub const TASK_PATH: &str = "/tasks/:id";
pub const OAS_TASK_PATH: &str = "/tasks/{id}";

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            TASKS_PATH,
            get(get_tasks::handler).post(create_task::handler),
        )
        .route(
            TASK_PATH,
            put(update_task::handler).delete(delete_task::handler),
        )
}
