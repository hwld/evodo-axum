use axum::{
    routing::{get, put},
    Router,
};

use crate::AppState;

pub mod create_task_node;
pub mod get_task_nodes;
pub mod update_task_node_info;

pub const TAG: &str = "task_node";

pub const TASK_NODES_PATH: &str = "/task-nodes";
pub const TASK_NODE_INFO_LIST_PATH: &str = "/task-node-info";
pub const TASK_NODE_INFO_PATH: &str = "/task-node-info/:id";
pub const OAS_TASK_NODE_INFO_PATH: &str = "/task-node-info/{id}";

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            TASK_NODES_PATH,
            get(get_task_nodes::handler).post(create_task_node::handler),
        )
        .route(TASK_NODE_INFO_PATH, put(update_task_node_info::handler))
}
