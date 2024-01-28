use axum::{
    extract::Query,
    response::{IntoResponse, Redirect},
};
use axum_login::{tower_sessions::Session, AuthSession};
use http::StatusCode;
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{features::auth::Auth, AppResult};

pub const CSRF_STATE_KEY: &str = "auth.state";
pub const NONCE_KEY: &str = "auth.nonce";
pub const AFTER_LOGIN_REDIRECT_KEY: &str = "auth.after-login-redirect-key";

#[derive(Debug, Deserialize, IntoParams)]
pub struct LoginRedirectsQuery {
    /// ログイン後にリダイレクトするページへのパス
    after_login_redirect: Option<String>,
}

#[tracing::instrument(err, skip(auth_session))]
#[utoipa::path(get, tag = super::TAG, path = super::Paths::login(), params(LoginRedirectsQuery))]
pub async fn handler(
    auth_session: AuthSession<Auth>,
    session: Session,
    Query(LoginRedirectsQuery {
        after_login_redirect,
    }): Query<LoginRedirectsQuery>,
) -> AppResult<impl IntoResponse> {
    let (auth_url, csrf_state, nonce) = auth_session.backend.authorize_url();

    // CLIENT_URLと合わせて使用するので`/`で始まっているかだけを確認する。
    let after_login_redirect = after_login_redirect.unwrap_or("/".into());
    if !after_login_redirect.starts_with('/') {
        return Ok(StatusCode::BAD_REQUEST.into_response());
    }

    session.insert(CSRF_STATE_KEY, csrf_state.secret()).await?;
    session.insert(NONCE_KEY, nonce.secret()).await?;
    session
        .insert(AFTER_LOGIN_REDIRECT_KEY, after_login_redirect)
        .await?;

    Ok(Redirect::to(auth_url.as_str()).into_response())
}
