use crate::{app::AppState, features::auth::Auth};
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use axum_login::login_required;
pub mod connect_block_task;
pub mod connect_subtask;
pub mod create_task;
pub mod delete_task;
pub mod disconnect_block_task;
pub mod disconnect_subtask;
pub mod get_task;
pub mod get_tasks;
pub mod reconnect_block_task;
pub mod reconnect_subtask;
pub mod update_task;
pub mod update_task_status;

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

    pub fn update_task_status_base() -> String {
        "/update-status".into()
    }

    pub fn update_task_status() -> String {
        Self::task() + &Self::update_task_status_base()
    }

    pub fn update_task_status_open_api() -> String {
        Self::task_open_api() + &Self::update_task_status_base()
    }

    pub fn subtask() -> String {
        "/subtask".into()
    }

    pub fn connect_subtask() -> String {
        Self::subtask() + "/connect"
    }

    pub fn reconnect_subtask() -> String {
        Self::subtask() + "/reconnect"
    }

    pub fn disconnect_subtask() -> String {
        Self::subtask() + "/disconnect"
    }

    pub fn block_task() -> String {
        "/block-task".into()
    }

    pub fn connect_block_task() -> String {
        Self::block_task() + "/connect"
    }

    pub fn reconnect_block_task() -> String {
        Self::block_task() + "/reconnect"
    }

    pub fn disconnect_block_task() -> String {
        Self::block_task() + "/disconnect"
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
            get(get_task::handler)
                .put(update_task::handler)
                .delete(delete_task::handler),
        )
        .route(
            &TaskPaths::connect_subtask(),
            post(connect_subtask::handler),
        )
        .route(
            &TaskPaths::reconnect_subtask(),
            put(reconnect_subtask::handler),
        )
        .route(
            &TaskPaths::disconnect_subtask(),
            delete(disconnect_subtask::handler),
        )
        .route(
            &TaskPaths::update_task_status(),
            put(update_task_status::handler),
        )
        .route(
            &TaskPaths::connect_block_task(),
            post(connect_block_task::handler),
        )
        .route(
            &TaskPaths::disconnect_block_task(),
            delete(disconnect_block_task::handler),
        )
        .route(
            &TaskPaths::reconnect_block_task(),
            put(reconnect_block_task::handler),
        )
        .route_layer(login_required!(Auth))
}
