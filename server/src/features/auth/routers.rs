pub mod login;
pub mod login_callback;
pub mod session;
pub mod signup;
pub mod signup_session;
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
        .route("/signup-session", get(signup_session::handler))
        .route("/session", get(session::handler))
}
