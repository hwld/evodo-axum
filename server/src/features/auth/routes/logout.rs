use crate::app::AppResult;
use crate::features::auth::Auth;
use axum::response::IntoResponse;
use axum_login::AuthSession;
use http::StatusCode;

#[tracing::instrument(err)]
#[utoipa::path(
    post,
    tag = super::TAG,
    path = super::AuthPaths::logout(),
    responses ((status = 200))
)]
pub async fn handler(mut auth_session: AuthSession<Auth>) -> AppResult<impl IntoResponse> {
    auth_session.logout().await?;

    Ok(StatusCode::OK.into_response())
}
