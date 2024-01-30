use crate::{app::AppState, features::auth::Auth};
use axum::{
    routing::{get, put},
    Router,
};
use axum_login::login_required;
pub mod create_task;
pub mod delete_task;
pub mod get_tasks;
pub mod update_task;

pub const TAG: &str = "task";

pub struct Paths;
impl Paths {
    pub fn tasks() -> String {
        "/tasks".into()
    }
    pub fn task() -> String {
        Self::tasks() + "/:id"
    }
    pub fn oas_task() -> String {
        Self::tasks() + "/{id}"
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            &Paths::tasks(),
            get(get_tasks::handler).post(create_task::handler),
        )
        .route(
            &Paths::task(),
            put(update_task::handler).delete(delete_task::handler),
        )
        .route_layer(login_required!(Auth))
}
