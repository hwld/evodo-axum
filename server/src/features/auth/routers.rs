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
    fn auth() -> String {
        "/auth".into()
    }

    pub fn login_callback() -> String {
        Paths::auth() + "/login-callback"
    }
    pub fn login() -> String {
        Paths::auth() + "/login"
    }
    pub fn session() -> String {
        Paths::auth() + "/session"
    }
    pub fn signup() -> String {
        Paths::auth() + "/signup"
    }
    pub fn signup_session() -> String {
        Paths::auth() + "/signup-session"
    }
    pub fn cancel_signup() -> String {
        Paths::auth() + "/cancel-signup"
    }
    pub fn logout() -> String {
        Paths::auth() + "/logout"
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
