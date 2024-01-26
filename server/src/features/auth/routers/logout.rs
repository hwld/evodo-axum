use axum::response::IntoResponse;
use axum_login::AuthSession;
use http::StatusCode;

use crate::{features::auth::Auth, AppResult};

#[tracing::instrument(err)]
#[utoipa::path(post, tag = super::TAG, path = super::LOGOUT_PATH, responses ((status = 200)))]
pub async fn handler(mut auth_session: AuthSession<Auth>) -> AppResult<impl IntoResponse> {
    auth_session.logout().await?;

    Ok(StatusCode::OK.into_response())
}
