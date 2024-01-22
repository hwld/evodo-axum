use axum::{
    routing::{get, put},
    Router,
};

pub mod create_task_node;
pub mod get_task_nodes;
pub mod update_task_node_info;

pub fn router() -> Router<sqlx::Pool<sqlx::Sqlite>> {
    Router::new()
        .route(
            "/task-nodes",
            get(get_task_nodes::handler).post(create_task_node::handler),
        )
        .route("/task-node-info/:id", put(update_task_node_info::handler))
}