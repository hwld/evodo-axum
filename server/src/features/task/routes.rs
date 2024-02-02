use crate::{app::AppState, features::auth::Auth};
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use axum_login::login_required;
pub mod connect_subtask;
pub mod create_task;
pub mod delete_task;
pub mod disconnect_subtask;
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

    pub fn task_open_api() -> String {
        Self::tasks() + "/{id}"
    }

    pub fn subtask() -> String {
        "/subtask".into()
    }

    pub fn connect_subtask() -> String {
        Self::subtask() + "/connect"
    }

    pub fn update_subtask() -> String {
        Self::subtask() + "/update"
    }

    pub fn disconnect_subtask() -> String {
        Self::subtask() + "/disconnect"
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
        .route(
            &TaskPaths::disconnect_subtask(),
            delete(disconnect_subtask::handle),
        )
        .route_layer(login_required!(Auth))
}
