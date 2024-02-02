use crate::{app::AppState, features::auth::Auth};
use axum::{
    routing::{get, post, put},
    Router,
};
use axum_login::login_required;
pub mod connect_subtask;
pub mod create_task;
pub mod delete_task;
pub mod get_tasks;
pub mod update_task;

pub const TAG: &str = "task";

pub struct TaskPaths;
impl TaskPaths {
    pub fn tasks() -> String {
        "/tasks".into()
    }
    pub fn task() -> String {
        Self::tasks() + "/:id"
    }
    pub fn oas_task() -> String {
        Self::tasks() + "/{id}"
    }
    pub fn subtasks() -> String {
        "/subtasks".into()
    }
    pub fn connect_subtask() -> String {
        Self::subtasks() + "/connect"
    }
    pub fn update_subtask() -> String {
        Self::subtasks() + "/update"
    }
    pub fn delete_subatsk() -> String {
        Self::subtasks() + "/delete"
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            &TaskPaths::tasks(),
            get(get_tasks::handler).post(create_task::handler),
        )
        .route(
            &TaskPaths::task(),
            put(update_task::handler).delete(delete_task::handler),
        )
        .route(
            &TaskPaths::connect_subtask(),
            post(connect_subtask::handler),
        )
        .route_layer(login_required!(Auth))
}
