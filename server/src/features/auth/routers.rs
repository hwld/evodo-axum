pub mod login;
pub mod login_callback;
use crate::Db;
use axum::{routing::get, Router};

pub fn router() -> Router<Db> {
    Router::new()
        .route("/login", get(login::handler))
        .route("/login-callback", get(login_callback::handler))
}
