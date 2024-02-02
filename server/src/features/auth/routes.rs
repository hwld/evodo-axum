pub mod cancel_signup;
pub mod login;
pub mod login_callback;
pub mod logout;
pub mod session;
pub mod signup;
pub mod signup_session;
use crate::app::AppState;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};

pub const TAG: &str = "auth";

pub struct AuthPaths;
impl AuthPaths {
    fn auth() -> String {
        "/auth".into()
    }

    pub fn login_callback() -> String {
        Self::auth() + "/login-callback"
    }
    pub fn login() -> String {
        Self::auth() + "/login"
    }
    pub fn session() -> String {
        Self::auth() + "/session"
    }
    pub fn signup() -> String {
        Self::auth() + "/signup"
    }
    pub fn signup_session() -> String {
        Self::auth() + "/signup-session"
    }
    pub fn cancel_signup() -> String {
        Self::auth() + "/cancel-signup"
    }
    pub fn logout() -> String {
        Self::auth() + "/logout"
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(&AuthPaths::login(), get(login::handler))
        .route(
            &AuthPaths::login_callback(),
            get(login_callback::handler)
                .layer(middleware::from_fn(login_callback::handle_all_error)),
        )
        .route(&AuthPaths::signup(), post(signup::handler))
        .route(&AuthPaths::signup_session(), get(signup_session::handler))
        .route(&AuthPaths::session(), get(session::handler))
        .route(&AuthPaths::cancel_signup(), post(cancel_signup::handler))
        .route(&AuthPaths::logout(), post(logout::handler))
}
