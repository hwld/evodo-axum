use crate::{app::AppState, features::auth::Auth};
use axum::{
    routing::{delete, post, put},
    Router,
};
use axum_login::login_required;
pub mod connect_block_task;
pub mod disconnect_block_task;
pub mod reconnect_block_task;

pub const TAG: &str = "block-task";

pub struct BlockTaskPaths;
impl BlockTaskPaths {
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
            &BlockTaskPaths::connect_block_task(),
            post(connect_block_task::handler),
        )
        .route(
            &BlockTaskPaths::disconnect_block_task(),
            delete(disconnect_block_task::handler),
        )
        .route(
            &BlockTaskPaths::reconnect_block_task(),
            put(reconnect_block_task::handler),
        )
        .route_layer(login_required!(Auth))
}
