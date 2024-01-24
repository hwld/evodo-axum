use axum::extract::Query;
use axum_login::{tower_sessions::Session, AuthSession};
use http::StatusCode;
use openidconnect::{CsrfToken, Nonce};
use serde::Deserialize;

use crate::{
    features::auth::{Auth, Credentials},
    AppResult,
};

use super::login::{CSRF_STATE_KEY, NONCE_KEY};

#[derive(Debug, Clone, Deserialize)]
pub struct AuthzResp {
    code: String,
    state: CsrfToken,
}

#[tracing::instrument(err, skip(auth_session, session))]
pub async fn handler(
    mut auth_session: AuthSession<Auth>,
    session: Session,
    Query(AuthzResp {
        code,
        state: new_state,
    }): Query<AuthzResp>,
) -> AppResult<StatusCode> {
    let Ok(Some(old_state)) = session.get::<CsrfToken>(CSRF_STATE_KEY).await else {
        return Ok(StatusCode::BAD_REQUEST);
    };

    let Ok(Some(nonce)) = session.get::<Nonce>(NONCE_KEY).await else {
        return Ok(StatusCode::BAD_REQUEST);
    };

    let creds = Credentials {
        code,
        old_state,
        new_state,
        nonce,
    };

    let user = match auth_session.authenticate(creds).await {
        Ok(Some(user)) => user,
        Ok(None) => return Ok(StatusCode::UNAUTHORIZED),
        Err(_) => return Ok(StatusCode::INTERNAL_SERVER_ERROR),
    };

    if auth_session.login(&user).await.is_err() {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(StatusCode::OK)
}
