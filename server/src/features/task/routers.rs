use crate::Db;
use axum::{
    routing::{get, put},
    Router,
};
pub mod create_task;
pub mod delete_task;
pub mod get_tasks;
pub mod update_task;

pub fn router() -> Router<Db> {
    Router::new()
        .route("/tasks", get(get_tasks::handler).post(create_task::handler))
        .route(
            "/tasks/:id",
            put(update_task::handler).delete(delete_task::handler),
        )
}
