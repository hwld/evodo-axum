pub mod login;
pub mod login_callback;
pub mod signup;
use crate::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", get(login::handler))
        .route("/login-callback", get(login_callback::handler))
        .route("/signup", post(signup::handler))
}
