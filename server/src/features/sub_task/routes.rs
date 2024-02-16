use crate::{app::AppState, features::auth::Auth};
use axum::{
    routing::{delete, post, put},
    Router,
};
use axum_login::login_required;
pub mod connect_sub_task;
pub mod disconnect_sub_task;
pub mod reconnect_sub_task;

pub const TAG: &str = "sub-task";

pub struct SubTaskPaths;
impl SubTaskPaths {
    pub fn sub_task() -> String {
        "/sub-task".into()
    }

    pub fn connect_sub_task() -> String {
        Self::sub_task() + "/connect"
    }

    pub fn reconnect_sub_task() -> String {
        Self::sub_task() + "/reconnect"
    }

    pub fn disconnect_sub_task() -> String {
        Self::sub_task() + "/disconnect"
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            &SubTaskPaths::connect_sub_task(),
            post(connect_sub_task::handler),
        )
        .route(
            &SubTaskPaths::reconnect_sub_task(),
            put(reconnect_sub_task::handler),
        )
        .route(
            &SubTaskPaths::disconnect_sub_task(),
            delete(disconnect_sub_task::handler),
        )
        .route_layer(login_required!(Auth))
}
