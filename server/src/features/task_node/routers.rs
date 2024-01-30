use axum::{
    routing::{get, put},
    Router,
};
use axum_login::login_required;

use crate::{app::AppState, features::auth::Auth};

pub mod create_task_node;
pub mod get_task_nodes;
pub mod update_task_node_info;

pub const TAG: &str = "task_node";

pub struct Paths;
impl Paths {
    pub fn task_nodes() -> String {
        "/task-nodes".into()
    }
    pub fn task_node_info_list() -> String {
        "/task-node-info".into()
    }
    pub fn task_node_info() -> String {
        Self::task_node_info_list() + "/:id"
    }
    pub fn oas_task_node_info() -> String {
        Self::task_node_info_list() + "/{id}"
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            &Paths::task_nodes(),
            get(get_task_nodes::handler).post(create_task_node::handler),
        )
        .route(
            &Paths::task_node_info(),
            put(update_task_node_info::handler),
        )
        .route_layer(login_required!(Auth))
}
