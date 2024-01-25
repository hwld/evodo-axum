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

pub const TAG: &str = "auth";

pub const LOGIN_CALLBACK_PATH: &str = "/login-callback";
pub const LOGIN_PATH: &str = "/login";
pub const SESSION_PATH: &str = "/session";
pub const SIGNUP_SESSION_PATH: &str = "/signup-session";
pub const SIGNUP_PATH: &str = "/signup";

pub fn router() -> Router<AppState> {
    Router::new()
        .route(LOGIN_PATH, get(login::handler))
        .route(LOGIN_CALLBACK_PATH, get(login_callback::handler))
        .route(SIGNUP_PATH, post(signup::handler))
        .route(SIGNUP_SESSION_PATH, get(signup_session::handler))
        .route(SESSION_PATH, get(session::handler))
}
