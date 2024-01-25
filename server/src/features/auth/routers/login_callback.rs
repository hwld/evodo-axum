use super::login::{CSRF_STATE_KEY, NONCE_KEY};
use crate::{
    features::auth::{Auth, AuthError, Credentials},
    AppResult,
};
use axum::{
    extract::Query,
    response::{IntoResponse, Redirect},
};
use axum_login::{tower_sessions::Session, AuthSession};
use http::StatusCode;
use openidconnect::{CsrfToken, Nonce};
use serde::Deserialize;

pub const SIGNUP_USER_ID: &str = "auth.signup.user-id";

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
) -> AppResult<impl IntoResponse> {
    let Ok(Some(old_state)) = session.get::<CsrfToken>(CSRF_STATE_KEY).await else {
        return Ok(StatusCode::BAD_REQUEST.into_response());
    };
    let Ok(Some(nonce)) = session.get::<Nonce>(NONCE_KEY).await else {
        return Ok(StatusCode::BAD_REQUEST.into_response());
    };

    let creds = Credentials {
        code,
        old_state,
        new_state,
        nonce,
    };

    let user = match auth_session.authenticate(creds).await {
        Ok(Some(user)) => user,
        // 認証は通っているがユーザーが存在しない場合は新規登録フローに移行させる
        Err(axum_login::Error::Backend(AuthError::AuthenticationUserNotFound(user_id))) => {
            session.insert(SIGNUP_USER_ID, user_id).await?;
            return Ok(Redirect::to("http://localhost:3000/signup").into_response());
        }
        Ok(None) => return Ok(StatusCode::UNAUTHORIZED.into_response()),
        Err(_) => {
            return Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response());
        }
    };

    auth_session.login(&user).await?;

    Ok(Redirect::to("http://localhost:3000/login").into_response())
}