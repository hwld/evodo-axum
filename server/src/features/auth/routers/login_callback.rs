use super::login::{CSRF_STATE_KEY, NONCE_KEY};
use crate::{
    features::auth::{Auth, AuthError, Credentials},
    AppResult,
};
use axum::{
    extract::{Query, Request},
    middleware::Next,
    response::{IntoResponse, Redirect},
};
use axum_login::{tower_sessions::Session, AuthSession};
use http::StatusCode;
use openidconnect::{CsrfToken, Nonce};
use serde::Deserialize;

/// 新規登録しようとしているユーザーのID
pub const SIGNUP_USER_ID_KEY: &str = "auth.signup.user-id";

#[derive(Debug, Clone, Deserialize)]
pub struct AuthzResp {
    code: String,
    state: CsrfToken,
}

#[tracing::instrument(err, skip(auth_session, session))]
#[utoipa::path(get, tag = super::TAG, path = super::LOGIN_CALLBACK_PATH)]
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
            // クリーンな新規登録セッションを作る
            session.flush().await?;
            session.insert(SIGNUP_USER_ID_KEY, user_id).await?;
            return Ok(Redirect::to("http://localhost:3000/signup").into_response());
        }
        Ok(None) => return Ok(StatusCode::UNAUTHORIZED.into_response()),
        Err(_) => {
            return Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response());
        }
    };

    auth_session.login(&user).await?;

    Ok(Redirect::to("http://localhost:3000").into_response())
}

/// このハンドラで発生したエラーはフロントエンド側で補足できないのでリダイレクトさせる
pub async fn handle_all_error(request: Request, next: Next) -> impl IntoResponse {
    let response = next.run(request).await;
    let status = response.status();

    if status.is_client_error() | status.is_server_error() {
        return Redirect::to("http://localhost:3000/auth-error").into_response();
    }

    response
}
