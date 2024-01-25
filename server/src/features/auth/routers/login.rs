use axum::response::{IntoResponse, Redirect};
use axum_login::{tower_sessions::Session, AuthSession};

use crate::{features::auth::Auth, AppResult};

// TODO
pub const CSRF_STATE_KEY: &str = "auth.state";
pub const NONCE_KEY: &str = "auth.nonce";

#[tracing::instrument(err, skip(auth_session))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    session: Session,
) -> AppResult<impl IntoResponse> {
    let (auth_url, csrf_state, nonce) = auth_session.backend.authorize_url();

    session.insert(CSRF_STATE_KEY, csrf_state.secret()).await?;
    session.insert(NONCE_KEY, nonce.secret()).await?;

    Ok(Redirect::to(auth_url.as_str()))
}
