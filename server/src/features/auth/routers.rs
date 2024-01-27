pub mod cancel_signup;
pub mod login;
pub mod login_callback;
pub mod logout;
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

pub struct Paths;
impl Paths {
    pub fn login_callback() -> String {
        "/login-callback".into()
    }
    pub fn login() -> String {
        "/login".into()
    }
    pub fn session() -> String {
        "/session".into()
    }
    pub fn signup() -> String {
        "/signup".into()
    }
    pub fn signup_session() -> String {
        "/signup-session".into()
    }
    pub fn cancel_signup() -> String {
        "/cancel-signup".into()
    }
    pub fn logout() -> String {
        "/logout".into()
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(&Paths::login(), get(login::handler))
        .route(
            &Paths::login_callback(),
            get(login_callback::handler)
                .layer(middleware::from_fn(login_callback::handle_all_error)),
        )
        .route(&Paths::signup(), post(signup::handler))
        .route(&Paths::signup_session(), get(signup_session::handler))
        .route(&Paths::session(), get(session::handler))
        .route(&Paths::cancel_signup(), post(cancel_signup::handler))
        .route(&Paths::logout(), post(logout::handler))
}
