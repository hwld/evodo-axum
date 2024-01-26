pub mod cancel_signup;
pub mod login;
pub mod login_callback;
pub mod session;
pub mod signup;
pub mod signup_session;
use crate::AppState;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};

pub const TAG: &str = "auth";

pub const LOGIN_CALLBACK_PATH: &str = "/login-callback";
pub const LOGIN_PATH: &str = "/login";
pub const SESSION_PATH: &str = "/session";
pub const SIGNUP_SESSION_PATH: &str = "/signup-session";
pub const SIGNUP_PATH: &str = "/signup";
pub const CANCEL_SIGNUP_PATH: &str = "/cancel-signup";

pub fn router() -> Router<AppState> {
    Router::new()
        .route(LOGIN_PATH, get(login::handler))
        .route(
            LOGIN_CALLBACK_PATH,
            get(login_callback::handler)
                .layer(middleware::from_fn(login_callback::handle_all_error)),
        )
        .route(SIGNUP_PATH, post(signup::handler))
        .route(SIGNUP_SESSION_PATH, get(signup_session::handler))
        .route(SESSION_PATH, get(session::handler))
        .route(CANCEL_SIGNUP_PATH, post(cancel_signup::handler))
}
